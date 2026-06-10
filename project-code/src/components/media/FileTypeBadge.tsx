interface FileTypeBadgeProps {
  filename: string;
  tagExt?: string | null;
}

export function FileTypeBadge({ filename, tagExt }: FileTypeBadgeProps) {
  const ext = tagExt?.toLowerCase() ?? filename.split(".").pop()?.toLowerCase() ?? "";

  const map: Record<string, { label: string; color: string; icon: string }> = {
    pdf:  { label: "PDF",  color: "#e53935", icon: "📄" },
    zip:  { label: "ZIP",  color: "#f57c00", icon: "🗜️" },
    rar:  { label: "RAR",  color: "#ef6c00", icon: "🗜️" },
    "7z": { label: "7Z",   color: "#e65100", icon: "🗜️" },
    tar:  { label: "TAR",  color: "#bf360c", icon: "🗜️" },
    gz:   { label: "GZ",   color: "#bf360c", icon: "🗜️" },
    doc:  { label: "DOC",  color: "#1565c0", icon: "📝" },
    docx: { label: "DOCX", color: "#1565c0", icon: "📝" },
    xls:  { label: "XLS",  color: "#2e7d32", icon: "📊" },
    xlsx: { label: "XLSX", color: "#2e7d32", icon: "📊" },
    ppt:  { label: "PPT",  color: "#c62828", icon: "📊" },
    pptx: { label: "PPTX", color: "#c62828", icon: "📊" },
    txt:  { label: "TXT",  color: "#546e7a", icon: "📃" },
    csv:  { label: "CSV", color: "#00695c", icon: "📃" },
    json: { label: "JSON", color: "#00838f", icon: "📃" },
    xml:  { label: "XML",  color: "#558b2f", icon: "📃" },
    vcf:  { label: "VCF",  color: "#6a1b9a", icon: "👤" },
    apk:  { label: "APK",  color: "#388e3c", icon: "📦" },
    exe:  { label: "EXE",  color: "#37474f", icon: "⚙️" },
    dmg:  { label: "DMG",  color: "#37474f", icon: "💿" },
    iso:  { label: "ISO",  color: "#455a64", icon: "💿" },
    gb:   { label: "GB",   color: "#7b1fa2", icon: "🎮" },
    gbc:  { label: "GBC",  color: "#7b1fa2", icon: "🎮" },
    gba:  { label: "GBA",  color: "#6a1b9a", icon: "🎮" },
    nds:  { label: "NDS",  color: "#4a148c", icon: "🎮" },
    n64:  { label: "N64",  color: "#4a148c", icon: "🎮" },
    z64:  { label: "N64",  color: "#4a148c", icon: "🎮" },
    v64:  { label: "N64",  color: "#4a148c", icon: "🎮" },
    sfc:  { label: "SNES", color: "#512da8", icon: "🎮" },
    smc:  { label: "SNES", color: "#512da8", icon: "🎮" },
    nes:  { label: "NES",  color: "#512da8", icon: "🎮" },
    gg:   { label: "GG",   color: "#7b1fa2", icon: "🎮" },
    sms:  { label: "SMS",  color: "#7b1fa2", icon: "🎮" },
    md:   { label: "GEN",  color: "#283593", icon: "🎮" },
    gen:  { label: "GEN",  color: "#283593", icon: "🎮" },
    pce:  { label: "PCE",  color: "#283593", icon: "🎮" },
    nsp:  { label: "NSP",  color: "#c62828", icon: "🎮" },
    xci:  { label: "XCI",  color: "#c62828", icon: "🎮" },
    cia:  { label: "CIA",  color: "#ad1457", icon: "🎮" },
    "3ds": { label: "3DS", color: "#c62828", icon: "🎮" },
    rom:  { label: "ROM",  color: "#4a148c", icon: "🎮" },
    svg:  { label: "SVG",  color: "#f9a825", icon: "🖼️" },
    tif:  { label: "TIFF", color: "#e65100", icon: "🖼️" },
    tiff: { label: "TIFF", color: "#e65100", icon: "🖼️" },
    avif: { label: "AVIF", color: "#7b1fa2", icon: "🖼️" },
    flv:  { label: "FLV",  color: "#e65100", icon: "🎬" },
    wmv:  { label: "WMV",  color: "#1565c0", icon: "🎬" },
    wma:  { label: "WMA",  color: "#1565c0", icon: "🎵" },
    mid:  { label: "MIDI", color: "#6a1b9a", icon: "🎵" },
    midi: { label: "MIDI", color: "#6a1b9a", icon: "🎵" },
    epub: { label: "EPUB", color: "#2e7d32", icon: "📚" },
    pages:   { label: "PAGES",   color: "#0d47a1", icon: "📝" },
    numbers: { label: "NUMBERS", color: "#1b5e20", icon: "📊" },
    key:     { label: "KEY",     color: "#b71c1c", icon: "📊" },
    odt:  { label: "ODT",  color: "#1565c0", icon: "📝" },
    ods:  { label: "ODS",  color: "#2e7d32", icon: "📊" },
    odp:  { label: "ODP",  color: "#c62828", icon: "📊" },
    rtf:  { label: "RTF",  color: "#37474f", icon: "📝" },
    ics:  { label: "ICS",  color: "#0288d1", icon: "📅" },
    html: { label: "HTML", color: "#e65100", icon: "🌐" },
    htm:  { label: "HTML", color: "#e65100", icon: "🌐" },
    yaml: { label: "YAML", color: "#546e7a", icon: "📃" },
    yml:  { label: "YAML", color: "#546e7a", icon: "📃" },
    toml: { label: "TOML", color: "#546e7a", icon: "📃" },
    sql:  { label: "SQL",  color: "#00695c", icon: "🗄️" },
    cab:  { label: "CAB",  color: "#bf360c", icon: "🗜️" },
    xz:   { label: "XZ",   color: "#bf360c", icon: "🗜️" },
    bz2:  { label: "BZ2",  color: "#bf360c", icon: "🗜️" },
    zst:  { label: "ZST",  color: "#bf360c", icon: "🗜️" },
  };

  const info = map[ext] ?? { label: ext.toUpperCase() || "FILE", color: "#607d8b", icon: "📎" };

  return (
    <span className="file-type-badge" style={{ background: info.color }} title={info.label}>
      <span className="file-type-badge-icon">{info.icon}</span>
      <span className="file-type-badge-label">{info.label}</span>
    </span>
  );
}
