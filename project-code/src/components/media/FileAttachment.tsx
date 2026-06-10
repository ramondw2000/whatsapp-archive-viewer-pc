import { useState, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { invoke } from "@tauri-apps/api/core";
import { useLazyVisibility, showModuleToast } from "../../LazyMediaImage";
import { MediaFallback } from "./MediaFallback";
import { FileTypeBadge } from "./FileTypeBadge";
import { GAME_EXTS, EXT_DESCRIPTIONS } from "../../constants/fileTypes";
import type { Message } from "../../types";

interface FileAttachmentInnerProps {
  msg: Message;
  idx: number;
  chatId: string;
  onTagSaved: (patch: Partial<Message>) => void;
  showToast: (message: string) => void;
}

function FileAttachmentInner({ msg, idx, chatId, onTagSaved, showToast }: FileAttachmentInnerProps) {
  const [mode, setMode] = useState<"idle"|"renaming">("idle");
  const [renameInput, setRenameInput] = useState(msg.display_name ?? msg.media ?? "");
  const [showInfo, setShowInfo] = useState(false);
  const [filePath, setFilePath] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [vcardError, setVcardError] = useState<string | null>(null);
  const [showVcardModal, setShowVcardModal] = useState(false);
  const [vcardContact, setVcardContact] = useState<{ name: string | null; phones: string[]; emails: string[] } | null>(null);

  const filename = msg.media ?? "";
  const displayName = msg.display_name ?? filename;
  const [fileExists, setFileExists] = useState(true);
  const [existsInZip, setExistsInZip] = useState(false);

  useEffect(() => {
    if (!chatId || !filename) return;

    setFileExists(true);
    setExistsInZip(false);

    const cleanFilename = filename.replace(/[\u200E\u200F\u202A-\u202E\u2066-\u2069\uFEFF\u200B]/g, "");
    invoke<string>("get_media_path", { chatId, filename: cleanFilename })
      .then(() => setFileExists(true))
      .catch(async () => {
        setFileExists(false);
        try {
          const inZip = await invoke<boolean>("check_file_in_zip", { chatId, filename });
          setExistsInZip(inZip);
        } catch { /* ignore */ }
      });
  }, [chatId, filename]);

  function extOf(name: string): string {
    const i = name.lastIndexOf(".");
    return (i > 0 && i < name.length - 1) ? name.slice(i + 1).toLowerCase() : "";
  }

  const effectiveExt = extOf(displayName) || extOf(filename) || (msg.tag_ext ?? "");
  const hasExt = !!extOf(filename);
  const isGame = GAME_EXTS.has(effectiveExt);
  const desc = EXT_DESCRIPTIONS[effectiveExt];

  function saveRename() {
    const newName = renameInput.trim();
    const newExt = extOf(newName);
    const i = filename.lastIndexOf(".");
    const baseName = (i > 0) ? filename.slice(0, i) : filename;
    const newFilename = newExt ? `${baseName}.${newExt}` : baseName;

    const promises = [
      invoke("set_display_name", { chatId, messageIdx: idx, displayName: newName || null }),
      invoke("set_file_tag", { chatId, messageIdx: idx, tagExt: newExt }),
    ];

    if (newFilename !== filename) {
      promises.push(invoke("rename_media_file", { chatId, messageIdx: idx, oldFilename: filename, newFilename }));
    }

    Promise.all(promises)
      .then(() => {
        onTagSaved({ display_name: newName || null, tag_ext: newExt || null, media: newFilename });
        showToast(`File renamed to "${newName}" - Change saved!`);
        setMode("idle");
      })
      .catch((e) => showModuleToast("Could not rename: " + e));
  }

  function openVcard() {
    setShowVcardModal(true);
  }

  function openVcardWithMethod(method: "app" | "web" | "default" | "view") {
    if (method === "view") {
      invoke<{ name: string | null; phones: string[]; emails: string[] }>("parse_vcard", { chatId, filename })
        .then(contact => {
          setVcardContact(contact);
          setShowVcardModal(false);
        })
        .catch((e) => {
          setVcardError("Failed to parse vCard: " + e);
          setShowVcardModal(false);
        });
    } else if (method === "default") {
      invoke("open_media_file", { chatId, filename })
        .then(() => setShowVcardModal(false))
        .catch((e) => {
          setVcardError("Failed to open file: " + e);
          setShowVcardModal(false);
        });
    } else {
      invoke("open_vcard_whatsapp", { chatId, filename, method })
        .then(() => setShowVcardModal(false))
        .catch((e) => {
          setVcardError("Failed to open WhatsApp: " + e);
          setShowVcardModal(false);
        });
    }
  }

  function openInfo() {
    if (!filePath) {
      invoke<string>("get_media_path", { chatId, filename })
        .then(p => setFilePath(p))
        .catch(() => setFilePath("(file not found)"));
    }
    setShowInfo(v => !v);
  }

  function copyPath() {
    if (filePath) {
      navigator.clipboard.writeText(filePath);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    }
  }

  if (!fileExists) return <MediaFallback filename={filename} chatId={chatId} existsInZip={existsInZip} />;

  return (
    <div className="file-attachment-wrapper">
      <div className="media-content file-attachment">
        <FileTypeBadge filename={filename} tagExt={effectiveExt || null} />
        <span className="file-attachment-name" title={filename}>{displayName}</span>
        <span className="file-attachment-actions">
          <button className="file-tag-btn" title="File info" onClick={openInfo}>ℹ️</button>
          {(!hasExt || msg.tag_ext || msg.display_name) && (
            <button
              className="file-tag-btn"
              title="Rename and tag file type"
              onClick={() => { setRenameInput(msg.display_name ?? filename); setMode("renaming"); }}
            >✏️</button>
          )}
          {hasExt && !isGame && (
            <button
              className="file-open-btn"
              title={effectiveExt === "vcf" ? "Add to WhatsApp" : "Open with default app"}
              onClick={() => {
                if (effectiveExt === "vcf") {
                  openVcard();
                } else {
                  invoke("open_media_file", { chatId, filename })
                    .catch((e) => showModuleToast("Could not open file: " + e));
                }
              }}
            >{effectiveExt === "vcf" ? "Add Contact" : "Open"}</button>
          )}
        </span>
      </div>
      {mode === "renaming" && (
        <div className="file-inline-edit">
          <span className="file-inline-label">
            {hasExt ? "Display name:" : "Rename (include .ext to tag type):"}
          </span>
          <input
            className="file-tag-input"
            style={{ minWidth: 0, flex: 1 }}
            value={renameInput}
            onChange={e => setRenameInput(e.target.value)}
            onKeyDown={e => { if (e.key === "Enter") saveRename(); if (e.key === "Escape") setMode("idle"); }}
            placeholder={hasExt ? filename : "e.g. my-game.gb"}
            autoFocus
          />
          <button className="file-open-btn" onClick={saveRename}>Save</button>
          <button className="file-tag-cancel" onClick={() => setMode("idle")}>✕</button>
        </div>
      )}
      {showInfo && (
        <div className={`file-info-popup${isGame ? " file-info-popup--game" : ""}`}>
          {isGame && <div className="file-info-game-header">🎮 Game File</div>}
          {desc && <div className="file-info-desc">{desc}</div>}
          <div className="file-info-path-row">
            <span className="file-info-path">{filePath ?? "Loading…"}</span>
            <button className="file-tag-btn" title="Copy path" onClick={copyPath}>
              {copied ? "✓" : "📋"}
            </button>
          </div>
        </div>
      )}
      {vcardError && (
        <div className="file-info-popup file-info-popup--error">
          <div className="file-info-error-msg">{vcardError}</div>
          <div className="file-info-error-actions">
            <button className="file-open-btn" onClick={() => { setVcardError(null); openVcard(); }}>Retry</button>
            <button className="file-tag-cancel" onClick={() => setVcardError(null)}>Dismiss</button>
          </div>
        </div>
      )}
      {showVcardModal && createPortal(
        <div className="vcard-modal-overlay" onClick={() => setShowVcardModal(false)}>
          <div className="vcard-modal" onClick={(e) => e.stopPropagation()}>
            <div className="vcard-modal-title">Open Contact</div>
            <div className="vcard-modal-desc">Choose how to open this vCard:</div>
            <div className="vcard-modal-buttons">
              <button className="vcard-modal-btn vcard-modal-btn--app" onClick={() => openVcardWithMethod("app")}>
                📱 WhatsApp App
              </button>
              <button className="vcard-modal-btn vcard-modal-btn--web" onClick={() => openVcardWithMethod("web")}>
                🌐 WhatsApp Web
              </button>
              <button className="vcard-modal-btn vcard-modal-btn--default" onClick={() => openVcardWithMethod("default")}>
                📄 Open with default app
              </button>
              <button className="vcard-modal-btn vcard-modal-btn--view" onClick={() => openVcardWithMethod("view")}>
                👁️ View contact info
              </button>
            </div>
            <button className="vcard-modal-cancel" onClick={() => setShowVcardModal(false)}>Cancel</button>
          </div>
        </div>
      , document.body)}
      {vcardContact && (
        <div className="file-info-popup file-info-popup--vcard">
          <div className="vcard-contact-info">
            {vcardContact.name && <div className="vcard-contact-name">👤 {vcardContact.name}</div>}
            {vcardContact.phones.length > 0 && (
              <div className="vcard-contact-section">
                <div className="vcard-contact-label">📱 Phone{vcardContact.phones.length > 1 ? 's' : ''}:</div>
                {vcardContact.phones.map((phone, i) => (
                  <div key={i} className="vcard-contact-value">{phone}</div>
                ))}
              </div>
            )}
            {vcardContact.emails.length > 0 && (
              <div className="vcard-contact-section">
                <div className="vcard-contact-label">✉️ Email{vcardContact.emails.length > 1 ? 's' : ''}:</div>
                {vcardContact.emails.map((email, i) => (
                  <div key={i} className="vcard-contact-value">{email}</div>
                ))}
              </div>
            )}
          </div>
          <button className="file-tag-cancel" onClick={() => setVcardContact(null)}>Close</button>
        </div>
      )}
    </div>
  );
}

interface FileAttachmentProps extends FileAttachmentInnerProps {
  className?: string;
}

export function FileAttachment(props: FileAttachmentProps) {
  const { ref, isVisible, style, className } = useLazyVisibility();

  return (
    <div ref={ref} style={style} className={className}>
      {isVisible ? <FileAttachmentInner {...props} /> : (
        <div className="media-skeleton" style={{ width: 250, minHeight: 180 }}><div className="skeleton-shimmer"></div></div>
      )}
    </div>
  );
}
