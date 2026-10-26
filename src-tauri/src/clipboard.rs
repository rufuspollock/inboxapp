//! Native clipboard selection. File URLs take priority over image representations
//! so copying an image file preserves its original bytes and filename.

use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Contents {
    Files(Vec<PathBuf>),
    Image {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    Png(Vec<u8>),
    Text(String),
    Empty,
}

pub fn read() -> Result<Contents, String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| format!("Cannot access clipboard: {error}"))?;
    match clipboard.get().file_list() {
        Ok(files) if !files.is_empty() => return Ok(Contents::Files(files)),
        Ok(_) | Err(arboard::Error::ContentNotAvailable) => {}
        Err(error) => return Err(format!("Cannot read clipboard files: {error}")),
    }

    // Read an explicitly offered PNG before asking AppKit to synthesize TIFF
    // for arboard. This preserves PNG bytes and checks their size before decode.
    if let Some(bytes) = read_png()? {
        return Ok(Contents::Png(bytes));
    }

    match clipboard.get_image() {
        Ok(image) => {
            let width = u32::try_from(image.width)
                .map_err(|_| "Clipboard image is too wide".to_string())?;
            let height = u32::try_from(image.height)
                .map_err(|_| "Clipboard image is too tall".to_string())?;
            return Ok(Contents::Image {
                width,
                height,
                rgba: image.bytes.into_owned(),
            });
        }
        Err(arboard::Error::ContentNotAvailable) => {}
        Err(error) => return Err(format!("Cannot read clipboard image: {error}")),
    }

    match clipboard.get_text() {
        Ok(text) if !text.is_empty() => Ok(Contents::Text(text)),
        Ok(_) | Err(arboard::Error::ContentNotAvailable) => Ok(Contents::Empty),
        Err(error) => Err(format!("Cannot read clipboard text: {error}")),
    }
}

#[cfg(target_os = "macos")]
fn read_png() -> Result<Option<Vec<u8>>, String> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSPasteboard;
    use objc2_foundation::NSString;

    autoreleasepool(|_| {
        let pasteboard = NSPasteboard::generalPasteboard();
        let declared_png = pasteboard
            .types()
            .is_some_and(|types| types.iter().any(|kind| kind.to_string() == "public.png"));
        if !declared_png {
            return Ok(None);
        }
        match pasteboard.dataForType(&NSString::from_str("public.png")) {
            Some(data) if data.len() > 20 * 1024 * 1024 => {
                Err("Clipboard image exceeds the 20 MB limit".into())
            }
            Some(data) => Ok(Some(data.to_vec())),
            None => Err("Cannot read the PNG advertised by the clipboard".into()),
        }
    })
}

#[cfg(not(target_os = "macos"))]
fn read_png() -> Result<Option<Vec<u8>>, String> {
    Ok(None)
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::ProtocolObject;
    use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
    use objc2_foundation::{NSArray, NSData, NSString};

    type Snapshot = Vec<Vec<(String, Vec<u8>)>>;

    // Materialize every item/type before changing anything. If a promised format
    // cannot be read, refuse to run rather than losing that representation.
    fn snapshot(pasteboard: &NSPasteboard) -> Result<Snapshot, String> {
        let before = pasteboard.changeCount();
        let mut result = Vec::new();
        if let Some(items) = pasteboard.pasteboardItems() {
            for item in items.iter() {
                let mut formats = Vec::new();
                for kind in item.types().iter() {
                    let bytes = item
                        .dataForType(&kind)
                        .ok_or_else(|| format!("Cannot preserve clipboard format {kind}"))?;
                    formats.push((kind.to_string(), bytes.to_vec()));
                }
                result.push(formats);
            }
        } else if pasteboard.types().is_some_and(|types| !types.is_empty()) {
            return Err("Cannot preserve clipboard items; leaving clipboard untouched".into());
        }
        if pasteboard.changeCount() != before {
            return Err("Clipboard changed while taking its snapshot".into());
        }
        Ok(result)
    }

    fn write(pasteboard: &NSPasteboard, items: &Snapshot) -> Result<(), String> {
        let mut objects: Vec<Retained<ProtocolObject<dyn NSPasteboardWriting>>> = Vec::new();
        // Build all replacement items before clearing the existing pasteboard.
        for formats in items {
            let item = NSPasteboardItem::new();
            for (kind, bytes) in formats {
                if !item.setData_forType(&NSData::with_bytes(bytes), &NSString::from_str(kind)) {
                    return Err(format!("Cannot write clipboard format {kind}"));
                }
            }
            objects.push(ProtocolObject::from_retained(item));
        }
        pasteboard.clearContents();
        if !objects.is_empty() && !pasteboard.writeObjects(&NSArray::from_retained_slice(&objects))
        {
            return Err("Cannot restore clipboard items".into());
        }
        Ok(())
    }

    struct RestoreClipboard {
        pasteboard: Retained<NSPasteboard>,
        saved: Snapshot,
    }

    impl RestoreClipboard {
        fn capture(pasteboard: Retained<NSPasteboard>) -> Result<Self, String> {
            let saved = snapshot(&pasteboard)?;
            Ok(Self { pasteboard, saved })
        }
    }

    impl Drop for RestoreClipboard {
        fn drop(&mut self) {
            // Runs during assertion unwinding too. Do not panic a second time.
            if let Err(error) = write(&self.pasteboard, &self.saved) {
                eprintln!("Clipboard restoration failed: {error}");
            }
        }
    }

    fn png() -> Vec<u8> {
        let mut bytes = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 255]).unwrap(),
        )
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn snapshot_restores_every_item_and_format_after_panic() {
        autoreleasepool(|_| {
            // A private pasteboard exercises the guard without touching the user.
            let pasteboard = NSPasteboard::pasteboardWithUniqueName();
            let original = vec![
                vec![
                    ("public.utf8-plain-text".into(), b"first".to_vec()),
                    ("public.html".into(), b"<b>first</b>".to_vec()),
                ],
                vec![("com.inboxapp.test-binary".into(), vec![0, 255, 3])],
            ];
            write(&pasteboard, &original).unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _restore = RestoreClipboard::capture(pasteboard.clone()).unwrap();
                pasteboard.clearContents();
                panic!("simulate failing native assertion");
            }));
            assert!(result.is_err());
            assert_eq!(snapshot(&pasteboard).unwrap(), original);
            pasteboard.clearContents();
        });
    }

    #[test]
    #[ignore = "opt-in: briefly changes the real macOS clipboard; restores all original items/formats"]
    fn native_clipboard_roundtrip() {
        autoreleasepool(|_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let restore = RestoreClipboard::capture(pasteboard.clone()).unwrap();
            let mut native = arboard::Clipboard::new().unwrap();
            let directory = tempfile::tempdir().unwrap();
            let pdf = directory.path().join("test document.pdf");
            let binary = directory.path().join("unknown.asset");
            std::fs::write(&pdf, b"%PDF-fixture").unwrap();
            std::fs::write(&binary, [0, 1, 255]).unwrap();
            // NSURL normalizes /var to /private/var on macOS.
            let pdf = pdf.canonicalize().unwrap();
            let binary = binary.canonicalize().unwrap();

            native.set().file_list(&[&pdf, &binary]).unwrap();
            assert_eq!(read().unwrap(), Contents::Files(vec![pdf, binary]));

            let bytes = png();
            write(
                &pasteboard,
                &vec![vec![("public.png".into(), bytes.clone())]],
            )
            .unwrap();
            assert_eq!(read().unwrap(), Contents::Png(bytes));

            // arboard writes an NSImage, whose native representation is TIFF.
            native
                .set_image(arboard::ImageData {
                    width: 1,
                    height: 1,
                    bytes: std::borrow::Cow::Owned(vec![255, 0, 0, 255]),
                })
                .unwrap();
            assert_eq!(
                read().unwrap(),
                Contents::Image {
                    width: 1,
                    height: 1,
                    rgba: vec![255, 0, 0, 255]
                }
            );

            native.set_text("plain clipboard text").unwrap();
            assert_eq!(
                read().unwrap(),
                Contents::Text("plain clipboard text".into())
            );

            // A corrupt advertised image must not silently become ordinary text.
            write(
                &pasteboard,
                &vec![vec![
                    ("public.tiff".into(), b"not a TIFF".to_vec()),
                    ("public.utf8-plain-text".into(), b"image label".to_vec()),
                ]],
            )
            .unwrap();
            assert!(read().unwrap_err().contains("Cannot read clipboard image"));
            native.clear().unwrap();
            assert_eq!(read().unwrap(), Contents::Empty);

            let original = restore.saved.clone();
            drop(restore);
            // Never print the user's clipboard if restoration verification fails.
            assert!(
                snapshot(&pasteboard).unwrap() == original,
                "Original clipboard was not restored"
            );
        });
    }

    #[test]
    #[ignore = "manual UI fixture: copies a PDF, waits for Enter, then restores the original clipboard"]
    fn native_ui_pdf_fixture() {
        use std::io::Write;

        autoreleasepool(|_| {
            let pasteboard = NSPasteboard::generalPasteboard();
            let restore = RestoreClipboard::capture(pasteboard.clone()).unwrap();
            let directory = tempfile::tempdir().unwrap();
            let pdf = directory.path().join("Inbox native PDF fixture.pdf");

            // A complete one-page PDF, including exact byte offsets and xref,
            // so the pasted attachment can be opened by Preview as well as saved.
            let stream = "BT /F1 18 Tf 72 720 Td (Inbox native attachment test) Tj ET\n";
            let objects = [
                "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
                "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
                format!("<< /Length {} >>\nstream\n{stream}endstream", stream.len()),
                "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
            ];
            let mut bytes = b"%PDF-1.4\n".to_vec();
            let mut offsets = Vec::new();
            for (index, object) in objects.iter().enumerate() {
                offsets.push(bytes.len());
                bytes.extend_from_slice(
                    format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes(),
                );
            }
            let xref = bytes.len();
            bytes.extend_from_slice(b"xref\n0 6\n0000000000 65535 f \n");
            for offset in offsets {
                bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
            }
            bytes.extend_from_slice(
                format!("trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n")
                    .as_bytes(),
            );
            std::fs::write(&pdf, &bytes).unwrap();
            let pdf = pdf.canonicalize().unwrap();
            let mut native = arboard::Clipboard::new().unwrap();
            native.set().file_list(&[&pdf]).unwrap();
            assert_eq!(read().unwrap(), Contents::Files(vec![pdf.clone()]));

            println!("PDF fixture: {}", pdf.display());
            println!("PDF ready; press Enter to restore clipboard");
            std::io::stdout().flush().unwrap();
            let mut input = String::new();
            assert!(
                std::io::stdin().read_line(&mut input).unwrap() > 0,
                "Run this manual fixture with an interactive stdin"
            );

            let original = restore.saved.clone();
            drop(restore);
            assert!(
                snapshot(&pasteboard).unwrap() == original,
                "Original clipboard was not restored"
            );
            println!("Original clipboard restored");
        });
    }
}
