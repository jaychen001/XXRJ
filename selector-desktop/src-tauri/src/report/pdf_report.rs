use std::fs;
use std::path::Path;

use super::content::report_lines;
use super::models::ReportPayload;

const PAGE_LINES: usize = 42;

pub fn write_pdf(path: &Path, payload: &ReportPayload) -> Result<(), String> {
    let lines = report_lines(payload);
    let pdf = build_pdf_bytes(&lines);
    fs::write(path, pdf).map_err(|error| error.to_string())
}

pub fn build_pdf_bytes(lines: &[String]) -> Vec<u8> {
    let pages = if lines.is_empty() {
        vec![vec!["空报告".to_string()]]
    } else {
        lines
            .chunks(PAGE_LINES)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>()
    };
    let mut objects = Vec::<(usize, String)>::new();
    let page_ids = (0..pages.len()).map(|index| 6 + index * 2).collect::<Vec<_>>();
    let content_ids = (0..pages.len()).map(|index| 7 + index * 2).collect::<Vec<_>>();

    objects.push((1, "<< /Type /Catalog /Pages 2 0 R >>".to_string()));
    objects.push((
        2,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            page_ids
                .iter()
                .map(|id| format!("{id} 0 R"))
                .collect::<Vec<_>>()
                .join(" "),
            page_ids.len()
        ),
    ));
    objects.push((3, "<< /Type /Font /Subtype /Type0 /BaseFont /STSong-Light /Encoding /UniGB-UCS2-H /DescendantFonts [4 0 R] >>".to_string()));
    objects.push((4, "<< /Type /Font /Subtype /CIDFontType0 /BaseFont /STSong-Light /CIDSystemInfo << /Registry (Adobe) /Ordering (GB1) /Supplement 2 >> /FontDescriptor 5 0 R >>".to_string()));
    objects.push((5, "<< /Type /FontDescriptor /FontName /STSong-Light /Flags 6 /FontBBox [0 -200 1000 900] /ItalicAngle 0 /Ascent 880 /Descent -120 /CapHeight 880 /StemV 80 >>".to_string()));

    for (index, page_lines) in pages.iter().enumerate() {
        objects.push((
            page_ids[index],
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
                content_ids[index]
            ),
        ));
        let stream = page_stream(page_lines);
        objects.push((
            content_ids[index],
            format!("<< /Length {} >>\nstream\n{}\nendstream", stream.as_bytes().len(), stream),
        ));
    }

    let max_id = objects.iter().map(|(id, _)| *id).max().unwrap_or(1);
    let mut bytes = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0usize; max_id + 1];
    for (id, object) in objects {
        offsets[id] = bytes.len();
        bytes.extend_from_slice(format!("{id} 0 obj\n{object}\nendobj\n").as_bytes());
    }
    let xref_offset = bytes.len();
    bytes.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", max_id + 1).as_bytes());
    for offset in offsets.iter().skip(1) {
        bytes.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    bytes.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
            max_id + 1,
            xref_offset
        )
        .as_bytes(),
    );
    bytes
}

fn page_stream(lines: &[String]) -> String {
    let mut stream = "BT\n/F1 11 Tf\n15 TL\n50 790 Td\n".to_string();
    for line in lines {
        for wrapped in wrap_line(line, 42) {
            stream.push_str(&format!("<{}> Tj\nT*\n", utf16be_hex(&wrapped)));
        }
    }
    stream.push_str("ET");
    stream
}

fn wrap_line(line: &str, limit: usize) -> Vec<String> {
    if line.is_empty() {
        return vec![" ".to_string()];
    }
    let chars = line.chars().collect::<Vec<_>>();
    chars
        .chunks(limit)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect()
}

fn utf16be_hex(value: &str) -> String {
    value
        .encode_utf16()
        .flat_map(|code| [(code >> 8) as u8, (code & 0xff) as u8])
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>()
}
