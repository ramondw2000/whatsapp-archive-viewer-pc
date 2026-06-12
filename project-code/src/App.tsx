import React, { useState, useEffect, useRef, Component } from 'react';
import { createPortal } from 'react-dom';


import type { ReactNode } from "react";


import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";


import { LazyMediaImage, useLazyVisibility, setMediaFallback, setToastCallback, showModuleToast } from "./LazyMediaImage";


import { VirtualMessageList, VirtualMessageListRef } from "./VirtualMessageList";


import { type Message, type SearchFilters, type ChatMeta, type ChatData, type SearchResult, type Profile, type SortOrder, type NameHistoryEntry, type BackgroundHistoryEntry, IMAGE_EXTS, VIDEO_EXTS, AUDIO_EXTS } from "./types";
import { formatDate } from "./utils/formatDate";
import { formatTime } from "./utils/formatTime";
import { createRenderMessageText, highlightText } from "./utils/textRendering";
import { LoadingOverlay } from "./components/LoadingOverlay";
import { JumpButton } from "./components/JumpButton";
import { LinkConfirmModal } from "./components/LinkConfirmModal";
import { ExpandableText } from "./components/ExpandableText";
import { DeleteConfirmationDialog } from "./components/dialogs/DeleteConfirmationDialog";
import { ExportSuccessDialog } from "./components/dialogs/ExportSuccessDialog";
import { SuccessNotificationDialog } from "./components/dialogs/SuccessNotificationDialog";
import { DeleteSuccessDialog } from "./components/dialogs/DeleteSuccessDialog";
import { UsernameDialog } from "./components/dialogs/UsernameDialog";
import { GroupParticipantsDialog } from "./components/dialogs/GroupParticipantsDialog";
import { ProfileDialog } from "./components/dialogs/ProfileDialog";
import { GroupAvatar } from "./components/GroupAvatar";
import { ProfileImage } from "./components/ProfileImage";
import { MediaFallback } from "./components/media/MediaFallback";
import { StickerImage } from "./components/media/StickerImage";
import { GifPlayer } from "./components/media/GifPlayer";
import { VideoPlayer } from "./components/media/VideoPlayer";
import { MediaImage } from "./components/media/MediaImage";
import { AudioBase64 } from "./components/media/AudioBase64";
import { FileTypeBadge } from "./components/media/FileTypeBadge";
import { FileAttachment } from "./components/media/FileAttachment";
import { GAME_EXTS } from "./constants/fileTypes";


import { VirtuosoGrid, Virtuoso } from "react-virtuoso";


import "./App.css";







class ErrorBoundary extends Component<


  { children: ReactNode },


  { error: string | null }


> {


  constructor(props: { children: ReactNode }) {


    super(props);


    this.state = { error: null };


  }


  static getDerivedStateFromError(err: unknown) {


    return { error: String(err) };


  }


  render() {


    if (this.state.error) {


      return (


        <div style={{ padding: 16, color: "red", fontFamily: "monospace", fontSize: 12 }}>


          <strong>Render error:</strong>


          <pre style={{ whiteSpace: "pre-wrap" }}>{this.state.error}</pre>


        </div>


      );


    }


    return this.props.children;


  }


}







function PollMessage({ content }: { content: string }) {


  const lines = content.split('\n');


  const header = lines[0]?.trim() || '';


  const name = lines[1]?.trim() || '';


  const options: { text: string; votes: string }[] = [];


  for (let i = 2; i < lines.length; i++) {


    const line = lines[i].trim();


    if (line.startsWith('OPTIE:') || line.startsWith('OPTION:')) {


      const match = line.replace(/^OPTIE:\s*/i, '').replace(/^OPTION:\s*/i, '').match(/^(.+?)\s*\((\d+\s*stemmen|\d+\s*votes)\)$/i);


      if (match) {


        options.push({ text: match[1].trim(), votes: match[2] });


      } else {


        options.push({ text: line.replace(/^OPTIE:\s*/i, '').replace(/^OPTION:\s*/i, ''), votes: '0 stemmen' });


      }


    }


  }


  return (


    <div className="poll-message">


      <div className="poll-header">{header}</div>


      <div className="poll-question">{name}</div>


      <div className="poll-options">


        {options.map((opt, i) => (


          <div key={i} className="poll-option">


            <span className="poll-option-text">{opt.text}</span>


            <span className="poll-option-votes">{opt.votes}</span>


          </div>


        ))}


      </div>


    </div>


  );


}


function EventMessage({ content }: { content: string }) {


  const lines = content.split('\n').map(l => l.trim()).filter(Boolean);


  const title = lines[0]?.replace(/^EVENEMENT:\s*/i, '').replace(/^EVENT:\s*/i, '') || '';


  const fields: { key: string; value: string }[] = [];


  for (let i = 1; i < lines.length; i++) {


    const sep = lines[i].indexOf(':');


    if (sep > 0) {


      const key = lines[i].slice(0, sep).trim();


      const value = lines[i].slice(sep + 1).trim();


      if (value) fields.push({ key, value });


    }


  }


  return (


    <div className="event-message">


      <div className="event-header">📅 {title}</div>


      <div className="event-fields">


        {fields.map((f, i) => (


          <div key={i} className="event-field">


            <span className="event-field-key">{f.key}:</span>


            <span className="event-field-value">{f.value}</span>


          </div>


        ))}


      </div>


    </div>


  );


}


function FileImageInlineInner({ chatId, filename, tagExt, onClick }: {


  chatId: string;


  filename: string;


  tagExt: string | null;


  onClick: () => void;


}) {


  const [src, setSrc] = useState<string | null>(null);


  useEffect(() => {


    if (!chatId || !filename) return;


    const MIME_MAP: Record<string, string> = {


      svg: "image/svg+xml", jpg: "image/jpeg", jpeg: "image/jpeg",


      png: "image/png", gif: "image/gif", webp: "image/webp",


      bmp: "image/bmp", heic: "image/heic", heif: "image/heic",


      tif: "image/tiff", tiff: "image/tiff", avif: "image/avif",


    };


    const cleanTag = (tagExt ?? "").trim().toLowerCase();


    const extFromFilename = filename.split(".").filter(Boolean).pop()?.toLowerCase() ?? "";


    const effectiveExt = cleanTag || extFromFilename;


    const mimeHint = MIME_MAP[effectiveExt] ?? null;


    const invokePayload = { chatId, filename, mimeHint };


    invoke<string>("get_media_as_base64", invokePayload)


      .then(b64 => {


        setSrc(b64);


      })


      .catch(err => {


        console.error("[FileImageInline] INVOKE ERROR:", err);


        setSrc(null);


      });


  }, [chatId, filename, tagExt]);


  return (


    <div className="media-content image" onClick={onClick} style={{ cursor: "pointer", display: "flex", justifyContent: "center", alignItems: "center", minHeight: "50px" }}>


      {src ? (


        <img src={src} alt={tagExt ?? filename} className="chat-image" style={{ maxWidth: "100%", maxHeight: "400px", display: "block" }} />


      ) : (


        <span style={{ color: "#888", fontSize: "12px" }}>Loading image...</span>


      )}


    </div>


  );


}


function FileImageInline(props: Parameters<typeof FileImageInlineInner>[0]) {


  const { ref, isVisible, style, className } = useLazyVisibility();


  return (


    <div ref={ref} style={style} className={className}>


      {isVisible ? <FileImageInlineInner {...props} /> : (


        <div className="media-skeleton" style={{ width: 250, minHeight: 180 }}><div className="skeleton-shimmer"></div></div>


      )}


    </div>


  );


}




type MediaTab = "images" | "videos" | "audio" | "files";


function MediaGallery({


  messages,


  chatId,


  onClose,


  onJumpToMessage,


  activeTab,


  onTabChange


}: {


  messages: Message[];


  chatId: string;


  onClose: () => void;


  onJumpToMessage: (index: number) => void;


  activeTab: MediaTab;


  onTabChange: (tab: MediaTab) => void;


}) {


  const [lightbox, setLightbox] = useState<{ filename: string; type: "image" | "video" | "gif"; sender: string; timestamp: string; index: number; hideControls?: boolean } | null>(null);


  const [sortOrder, setSortOrder] = useState<SortOrder>("newest");


  const [extFilter, setExtFilter] = useState<string>("all");


  const [vcardError, setVcardError] = useState<string | null>(null);


  const [showVcardModal, setShowVcardModal] = useState(false);


  const [vcardFilename, setVcardFilename] = useState<string>("");


  const [vcardContact, setVcardContact] = useState<{ name: string | null; phones: string[]; emails: string[] } | null>(null);


  function effectiveFileExt(m: Message): string {


    const extOf = (s: string) => { const i = s.lastIndexOf("."); return (i > 0 && i < s.length - 1) ? s.slice(i + 1).toLowerCase() : ""; };


    return extOf(m.display_name ?? "") || extOf(m.media ?? "") || (m.tag_ext ?? "");


  }


  function getExts(msgs: (Message & { _idx: number })[], useType?: boolean): string[] {


    const exts = new Set<string>();


    msgs.forEach(m => {


      if (useType) {


        exts.add(m.type);


      } else {


        const e = effectiveFileExt(m);


        if (e) exts.add(e);


      }


    });


    return Array.from(exts).sort();


  }


  function filterByExt<T extends { media?: string | null; type?: string }>(msgs: T[], useType?: boolean): T[] {


    if (extFilter === "all") return msgs;


    if (useType) return msgs.filter(m => (m as any).type === extFilter);


    return msgs.filter(m => effectiveFileExt(m as unknown as Message) === extFilter);


  }


  function switchTab(tab: MediaTab) {


    onTabChange(tab);


    setExtFilter("all");


  }


  function openVcard(filename: string) {


    setVcardFilename(filename);


    setShowVcardModal(true);


  }


  function openVcardWithMethod(method: "app" | "web" | "default" | "view") {


    if (method === "view") {


      // Parse and display contact info


      invoke<{ name: string | null; phones: string[]; emails: string[] }>("parse_vcard", { chatId, filename: vcardFilename })


        .then(contact => {


          setVcardContact(contact);


          setShowVcardModal(false);


        })


        .catch((e) => {


          setVcardError("Failed to parse vCard: " + e);


          setShowVcardModal(false);


        });


    } else if (method === "default") {


      // Open with default system app


      invoke("open_media_file", { chatId, filename: vcardFilename })


        .then(() => setShowVcardModal(false))


        .catch((e) => {


          setVcardError("Failed to open file: " + e);


          setShowVcardModal(false);


        });


    } else {


      // Open via WhatsApp


      invoke("open_vcard_whatsapp", { chatId, filename: vcardFilename, method })


        .then(() => setShowVcardModal(false))


        .catch((e) => {


          setVcardError("Failed to open WhatsApp: " + e);


          setShowVcardModal(false);


        });


    }


  }


  const sortFn = (a: Message & { _idx: number }, b: Message & { _idx: number }) =>


    sortOrder === "newest" ? b._idx - a._idx : a._idx - b._idx;


  const indexed = messages.map((m, i) => ({ ...m, _idx: i }));


  const imageMessages = indexed.filter(m => m.type === "image" && m.media).sort(sortFn);


  const audioMessages = indexed.filter(m => m.type === "audio" && m.media).sort(sortFn);


  const fileMessages  = indexed.filter(m => m.type === "file"  && m.media).sort(sortFn);


  const videoMessages = indexed.filter(m => m.type === "video" && m.media).sort(sortFn);


  const gifMessages   = indexed.filter(m => m.type === "gif"   && m.media).sort(sortFn);


  // Per-tab media for lightbox navigation (same sort as grid display)


  const imagesMedia = imageMessages;


  const videosMedia = [...videoMessages, ...gifMessages].sort(sortFn);


  function getTabMedia() {


    if (activeTab === "images") return imagesMedia;


    if (activeTab === "videos") return videosMedia;


    return [];


  }


  function goToPrev() {


    const tabMedia = getTabMedia();


    if (!lightbox || tabMedia.length <= 1 || lightbox.index <= 0) return;


    const newIndex = lightbox.index - 1;


    const msg = tabMedia[newIndex];


    setLightbox({ filename: msg.media!, type: msg.type as "image" | "video" | "gif", sender: msg.sender, timestamp: msg.timestamp, index: newIndex });


  }


  function goToNext() {


    const tabMedia = getTabMedia();


    if (!lightbox || tabMedia.length <= 1 || lightbox.index >= tabMedia.length - 1) return;


    const newIndex = lightbox.index + 1;


    const msg = tabMedia[newIndex];


    setLightbox({ filename: msg.media!, type: msg.type as "image" | "video" | "gif", sender: msg.sender, timestamp: msg.timestamp, index: newIndex });


  }


  // Keyboard navigation for lightbox


  useEffect(() => {


    if (!lightbox) return;


    function handleKey(e: KeyboardEvent) {


      if (e.key === "ArrowLeft") goToPrev();


      else if (e.key === "ArrowRight") goToNext();


      else if (e.key === "Escape") setLightbox(null);


    }


    window.addEventListener("keydown", handleKey);


    return () => window.removeEventListener("keydown", handleKey);


  }, [lightbox, activeTab, imagesMedia.length, videosMedia.length]);


  function formatFullDate(timestamp: string): string {


    const parts = timestamp.match(/(\d{1,2})[/\-](\d{1,2})[/\-](\d{2,4})/);


    if (!parts) return timestamp;


    return `${parts[1]}/${parts[2]}/${parts[3]}`;


  }


  return (


    <div className="media-gallery">


      <div className="media-gallery-header">


        <h3>Media, Links &amp; Docs</h3>


        <button className="media-gallery-close" onClick={onClose}>✕</button>


      </div>


      <div className="media-gallery-tabs">


        <button


          className={`media-tab-btn${activeTab === "images" ? " active" : ""}`}


          onClick={() => switchTab("images")}


        >


          Photos


          <span className="media-tab-count">{imageMessages.length}</span>


        </button>


        <button


          className={`media-tab-btn${activeTab === "videos" ? " active" : ""}`}


          onClick={() => switchTab("videos")}


        >


          Videos &amp; GIFs


          <span className="media-tab-count">{videoMessages.length + gifMessages.length}</span>


        </button>


        <button


          className={`media-tab-btn${activeTab === "audio" ? " active" : ""}`}


          onClick={() => switchTab("audio")}


        >


          Audio


          <span className="media-tab-count">{audioMessages.length}</span>


        </button>


        <button


          className={`media-tab-btn${activeTab === "files" ? " active" : ""}`}


          onClick={() => switchTab("files")}


        >


          Files


          <span className="media-tab-count">{fileMessages.length}</span>


        </button>


      </div>


      <div className="media-gallery-sort">


        <span className="media-sort-label">Sort:</span>


        <button


          className={`media-sort-btn${sortOrder === "newest" ? " active" : ""}`}


          onClick={() => setSortOrder("newest")}


        >Newest</button>


        <button


          className={`media-sort-btn${sortOrder === "oldest" ? " active" : ""}`}


          onClick={() => setSortOrder("oldest")}


        >Oldest</button>


        {(() => {


          const isVideoTab = activeTab === "videos";


          const pool =


            activeTab === "images" ? imageMessages :


            isVideoTab ? [...videoMessages, ...gifMessages] :


            activeTab === "audio"  ? audioMessages :


            fileMessages;


          const exts = getExts(pool, isVideoTab);


          return (


            <select


              className="media-ext-filter"


              value={extFilter}


              onChange={e => setExtFilter(e.target.value)}


            >


              <option value="all">All types</option>


              {exts.map(e => (


                <option key={e} value={e}>


                  {isVideoTab ? (e === "gif" ? "GIF" : `video (.${e})`) : `.${e}`}


                </option>


              ))}


            </select>


          );


        })()}


      </div>


      <div className="media-gallery-content">


        {activeTab === "images" && (


          filterByExt(imageMessages).length === 0 ? (


            <div className="media-empty">No photos</div>


          ) : (


            <VirtuosoGrid


              style={{ height: "100%" }}


              totalCount={filterByExt(imageMessages).length}


              listClassName="media-grid"


              itemClassName="media-grid-item-wrapper"


              itemContent={(i) => {


                const msg = filterByExt(imageMessages)[i];


                return (


                  <div


                    className="media-grid-item"


                    onClick={() => setLightbox({ filename: msg.media!, type: "image", sender: msg.sender, timestamp: msg.timestamp, index: imagesMedia.findIndex((m: Message & { _idx: number }) => m._idx === msg._idx) })}


                  >


                    <MediaImage


                      chatId={chatId}


                      filename={msg.media!}


                      onError={(e) => {


                        (e.target as HTMLImageElement).closest(".media-grid-item")?.classList.add("broken");


                      }}


                    />


                    <div className="media-grid-overlay">


                      <span className="media-grid-sender">{msg.sender}</span>


                      <span className="media-grid-date">{formatFullDate(msg.timestamp)}</span>


                    </div>


                    <button


                      className="media-jump-btn"


                      title="Jump to message"


                      onClick={(e) => { e.stopPropagation(); onClose(); onJumpToMessage(msg._idx); }}


                    >↗</button>


                  </div>


                );


              }}


            />


          )


        )}


        {activeTab === "videos" && (() => {


          const videoItems = filterByExt([...videoMessages, ...gifMessages], true).sort(sortFn);


          return videoItems.length === 0 ? (


            <div className="media-empty">No videos or GIFs</div>


          ) : (


            <VirtuosoGrid


              style={{ height: "100%" }}


              totalCount={videoItems.length}


              listClassName="media-grid"


              itemClassName="media-grid-item-wrapper"


              itemContent={(i) => {


                const msg = videoItems[i];


                return (


                  <div


                    className={`media-grid-item${msg.type === "gif" ? " media-grid-item--gif" : " media-grid-item--video"}`}


                    onClick={() => setLightbox({ filename: msg.media!, type: msg.type as "video" | "gif", sender: msg.sender, timestamp: msg.timestamp, index: videosMedia.findIndex((m: Message & { _idx: number }) => m._idx === msg._idx) })}


                  >


                    {msg.type === "gif" ? (


                      <GifPlayer chatId={chatId} index={i} filename={msg.media!} />


                    ) : (


                      <VideoPlayer chatId={chatId} index={i} filename={msg.media!} controls={false} className="media-grid-video" />


                    )}


                    <div className="media-grid-overlay">


                      <span className="media-grid-sender">{msg.sender}</span>


                      <span className="media-grid-date">{formatFullDate(msg.timestamp)}</span>


                    </div>


                    <button


                      className="media-jump-btn"


                      title="Jump to message"


                      onClick={(e) => { e.stopPropagation(); onClose(); onJumpToMessage(msg._idx); }}


                    >↗</button>


                  </div>


                );


              }}


            />


          );


        })()}


        {activeTab === "audio" && (() => {


          const audioItems = filterByExt(audioMessages);


          return audioItems.length === 0 ? (


            <div className="media-empty">No audio files</div>


          ) : (


            <Virtuoso


              style={{ height: "100%" }}


              totalCount={audioItems.length}


              components={{ List: React.forwardRef((props, ref) => <div {...props} ref={ref} className="media-audio-list" />) }}


              itemContent={(i) => {


                const msg = audioItems[i];


                return (


                  <div className="media-audio-item">


                    <div className="media-audio-info">


                      <span className="media-audio-icon">🎵</span>


                      <div>


                        <div className="media-audio-name">{msg.media}</div>


                        <div className="media-audio-meta">


                          {msg.sender} · {formatFullDate(msg.timestamp)} {msg.timestamp.match(/(\d{1,2}:\d{2})/)?.[1] ?? ""}


                          {msg.duration ? <span className="media-audio-duration"> · {msg.duration}</span> : null}


                        </div>


                      </div>


                    </div>


                    <div className="media-audio-controls">


                      <AudioBase64 chatId={chatId} filename={msg.media!} />


                      <button


                        className="media-jump-btn media-jump-btn--inline"


                        title="Jump to message"


                        onClick={() => { onClose(); onJumpToMessage(msg._idx); }}


                      >↗ Jump</button>


                    </div>


                  </div>


                );


              }}


            />


          );


        })()}


        {activeTab === "files" && (() => {


          const fileItems = filterByExt(fileMessages);


          return fileItems.length === 0 ? (


            <div className="media-empty">No files</div>


          ) : (


            <Virtuoso


              style={{ height: "100%" }}


              totalCount={fileItems.length}


              components={{ List: React.forwardRef((props, ref) => <div {...props} ref={ref} className="media-files-list" />) }}


              itemContent={(i) => {


                const msg = fileItems[i];


                return (


                  <div className="media-file-item">


                    <FileTypeBadge filename={msg.media!} tagExt={effectiveFileExt(msg) || null} />


                    <div className="media-file-info">


                      <div className="media-file-name" title={msg.media!}>{msg.display_name ?? msg.media}</div>


                      <div className="media-file-meta">{msg.sender} · {formatFullDate(msg.timestamp)} {msg.timestamp.match(/(\d{1,2}:\d{2})/)?.[1] ?? ""}</div>


                    </div>


                    {(() => { const ext = effectiveFileExt(msg); const isGame = GAME_EXTS.has(ext); const hasExt = !!ext; const isVcf = ext === "vcf"; return (hasExt && !isGame) ? (


                      <button


                        className="file-open-btn"


                        title={isVcf ? "Add to WhatsApp" : "Open with default app"}


                        onClick={() => {


                          if (isVcf) {


                            openVcard(msg.media!);


                          } else {


                            invoke("open_media_file", { chatId, filename: msg.media! })


                              .catch((e) => showModuleToast("Could not open file: " + e));


                          }


                        }}


                      >{isVcf ? "Add Contact" : "Open"}</button>


                    ) : null; })()}


                    <button


                      className="media-jump-btn media-jump-btn--inline"


                      title="Jump to message"


                      onClick={() => { onClose(); onJumpToMessage(msg._idx); }}


                    >↗ Jump</button>


                  </div>


                );


              }}


            />


          );


        })()}


        {vcardError && (


          <div className="file-info-popup file-info-popup--error" style={{ position: "fixed", bottom: 20, right: 20, zIndex: 1000 }}>


            <div className="file-info-error-msg">{vcardError}</div>


            <div className="file-info-error-actions">


              <button className="file-open-btn" onClick={() => { setVcardError(null); openVcard(vcardFilename); }}>Retry</button>


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


          <div className="file-info-popup file-info-popup--vcard" style={{ position: "fixed", bottom: 20, right: 20, zIndex: 1000 }}>


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


      {lightbox && (


        <div className="lightbox-overlay" onClick={() => setLightbox(null)}>


          <button className="lightbox-close" onClick={() => setLightbox(null)}>✕</button>


          {(() => {


            const tabMedia = getTabMedia();


            const atFirst = lightbox.index <= 0;


            const atLast = lightbox.index >= tabMedia.length - 1;


            return tabMedia.length > 1 && (


              <>


                <button className={`lightbox-nav lightbox-nav--prev${atFirst ? " disabled" : ""}`} onClick={(e) => { e.stopPropagation(); goToPrev(); }} disabled={atFirst}>‹</button>


                <button className={`lightbox-nav lightbox-nav--next${atLast ? " disabled" : ""}`} onClick={(e) => { e.stopPropagation(); goToNext(); }} disabled={atLast}>›</button>


              </>


            );


          })()}


          <div className="lightbox-meta" onClick={(e) => e.stopPropagation()}>


            <span className="lightbox-sender">{lightbox.sender}</span>


            <span className="lightbox-timestamp">{lightbox.timestamp}</span>


            {(() => {


              const tabMedia = getTabMedia();


              return !lightbox.hideControls && (


                <button


                  className="lightbox-fav-btn"


                  onClick={(e) => {


                    e.stopPropagation();


                    const currentMsg = tabMedia[lightbox.index];


                    if (currentMsg) {


                      const newFavorite = !currentMsg.is_favorite;


                      invoke("toggle_message_favorite", {


                        chatId: chatId,


                        messageIdx: currentMsg._idx,


                        isFavorite: newFavorite


                      }).catch(console.error);


                    }


                  }}


                  title={tabMedia[lightbox.index]?.is_favorite ? "Remove from favorites" : "Add to favorites"}


                >


                  <svg viewBox="0 0 24 24" width="24" height="24" fill={tabMedia[lightbox.index]?.is_favorite ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">


                    <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>


                  </svg>


                </button>


              );


            })()}


          </div>


          {lightbox.type === "image" ? (


            <MediaImage


              chatId={chatId}


              filename={lightbox.filename}


              alt=""


              className="lightbox-img"


            />


          ) : lightbox.type === "gif" ? (


            <div onClick={(e) => e.stopPropagation()}>


              <GifPlayer chatId={chatId} filename={lightbox.filename} />


            </div>


          ) : (


            <div onClick={(e) => e.stopPropagation()}>


              <VideoPlayer chatId={chatId} filename={lightbox.filename} controls className="lightbox-video" />


            </div>


          )}


        </div>


      )}


    </div>


  );


}


function StartScreen({ onOpenChats }: { onOpenChats: () => void }) {

  const [isCheckingUpdates, setIsCheckingUpdates] = useState(false);
  const [updateInfo, setUpdateInfo] = useState<{ version: string; url: string } | null>(null);
  const [updateMessage, setUpdateMessage] = useState<{ title: string; message: string } | null>(null);

  const handleCheckUpdates = async () => {
    setIsCheckingUpdates(true);
    setUpdateInfo(null);

    try {
      // Replace with your actual GitHub repo
      const repoOwner = "ramondw2000";
      const repoName = "whatsapp-archive-viewer-pc";
      const apiUrl = `https://api.github.com/repos/${repoOwner}/${repoName}/releases`;
      console.log("Fetching from:", apiUrl);
      const response = await fetch(apiUrl, {
        headers: {
          "Accept": "application/vnd.github.v3+json"
        }
      });
      console.log("GitHub API response status:", response.status, response.statusText);

      if (!response.ok) {
        const errorText = await response.text();
        console.error("GitHub API error:", errorText);
        setUpdateMessage({
          title: "No Releases",
          message: `API Error ${response.status}: ${response.statusText}. This feature will be available once releases are published.`
        });
        return;
      }

      const releases = await response.json();
      if (!releases || releases.length === 0) {
        setUpdateMessage({
          title: "No Releases",
          message: "No releases found. This feature will be available once releases are published."
        });
        return;
      }

      const data = releases[0]; // Get the most recent release (including prereleases)
      const latestVersion = data.tag_name.replace("v", "");
      const currentVersion = __APP_VERSION__;

      if (latestVersion !== currentVersion) {
        setUpdateInfo({
          version: latestVersion,
          url: data.html_url
        });
      } else {
        setUpdateMessage({
          title: "Up to Date",
          message: "You're already on the latest version"
        });
      }
    } catch (error) {
      console.error("Update check error:", error);
      setUpdateMessage({
        title: "Update Check Failed",
        message: `Error: ${error instanceof Error ? error.message : String(error)}`
      });
    } finally {
      setIsCheckingUpdates(false);
    }
  };

  return (
    <>
    <div className="start-screen">


      <div className="start-screen-header">
        {updateInfo ? (
          <button
            onClick={() => openUrl(updateInfo.url).catch(console.error)}
            className="check-updates-button update-available"
          >
            Update Available (v{updateInfo.version})
          </button>
        ) : (
          <button
            className="check-updates-button"
            onClick={handleCheckUpdates}
            disabled={isCheckingUpdates}
            title="Check for Updates"
          >
            {isCheckingUpdates ? "Checking..." : "Check for Updates"}
          </button>
        )}
      </div>


      <div className="start-screen-content">


        <img src="/icon.svg" alt="App Logo" className="start-screen-logo" />


        <h1 className="start-screen-title">WhatsApp Archive Viewer</h1>


        <button className="start-screen-button" onClick={onOpenChats}>


          Open Chat Overview


        </button>


      </div>


    </div>

    {updateMessage && (
      <div className="dialog-overlay" onClick={() => setUpdateMessage(null)}>
        <div className="dialog-content" onClick={e => e.stopPropagation()}>
          <button className="dialog-close-btn" onClick={() => setUpdateMessage(null)}>
            ×
          </button>
          <h2>{updateMessage.title}</h2>
          <p>{updateMessage.message}</p>
          <div className="dialog-buttons" style={{ justifyContent: "center" }}>
            <button className="dialog-btn-primary" onClick={() => setUpdateMessage(null)}>
              OK
            </button>
          </div>
        </div>
      </div>
    )}
    </>


  );


}


function App() {


  useEffect(() => { setMediaFallback(MediaFallback); }, []);


  const [chats, setChats] = useState<ChatMeta[]>([]);


  const [selectedChat, setSelectedChat] = useState<string | null>(null);
  const selectedChatRef = useRef<string | null>(null);
  // Keep ref in sync with state so async callbacks always see current value
  useEffect(() => { selectedChatRef.current = selectedChat; }, [selectedChat]);

  const isAndroid = /android/i.test(navigator.userAgent);
  const [isMobile, setIsMobile] = useState(() => isAndroid || window.innerWidth <= 900);

  useEffect(() => {
    const handler = () => setIsMobile(isAndroid || window.innerWidth <= 900);
    window.addEventListener('resize', handler);
    return () => window.removeEventListener('resize', handler);
  }, []);


  const [messages, setMessages] = useState<Message[]>([]);


  const [loading, setLoading] = useState(false);


  const [importing, setImporting] = useState(false);


  const [importProgress, setImportProgress] = useState<{ current: number; total: number } | null>(null);


  const [importDetail, setImportDetail] = useState<{ messages: number; media: number; phase: string } | null>(null);


  const [username, setUsername] = useState<string>("");


  const [showUsernameDialog, setShowUsernameDialog] = useState(false);


  const [theme, setTheme] = useState<"light" | "dark">("light");


  const [showMediaGallery, setShowMediaGallery] = useState(false);


  const [mediaGalleryTab, setMediaGalleryTab] = useState<MediaTab>("images");


  const [showVCDialog, setShowVCDialog] = useState(false);


  const [showStartScreen, setShowStartScreen] = useState(true);


  const [showFavorites, setShowFavorites] = useState(false);


  const [favoriteMessages, setFavoriteMessages] = useState<Message[]>([]);


  const [selectedMessageIndices, setSelectedMessageIndices] = useState<Set<number>>(new Set());


  const [_lastSelectedIndex, setLastSelectedIndex] = useState<number | null>(null);


  const [multiSelectMode, setMultiSelectMode] = useState(false);


  const [pressTimers, setPressTimers] = useState<Map<number, number>>(new Map());


  const [lastLongPressedIndex, setLastLongPressedIndex] = useState<number | null>(null);


  // Chat media lightbox state


  const [chatLightbox, setChatLightbox] = useState<{ filename: string; type: "image" | "video" | "gif"; sender: string; timestamp: string; index: number; hideControls?: boolean } | null>(null);


  const messagesEndRef = useRef<HTMLDivElement>(null);


  const messageRefs = useRef<(HTMLDivElement | null)[]>([]);


  const shouldScrollToBottom = useRef(false);


  // Scroll detection state


  const [_chatScrollState, setChatScrollState] = useState<{ atTop: boolean; atBottom: boolean; hasOverflow: boolean }>({ atTop: true, atBottom: true, hasOverflow: false });


  const messagesContainerRef = useRef<HTMLDivElement>(null);
  const touchScrollingRef = useRef(false);
  const touchStartPos = useRef<{ x: number; y: number } | null>(null);
  const loadCancelRef = useRef<{ cancelled: boolean } | null>(null);


  // Chat search state


  const [chatSearchQuery, setChatSearchQuery] = useState("");


  const [chatSearchResults, setChatSearchResults] = useState<ChatMeta[]>([]);


  const [isSearchingChats, setIsSearchingChats] = useState(false);


  // Message search state


  const [messageSearchQuery, setMessageSearchQuery] = useState("");


  const [messageSearchResults, setMessageSearchResults] = useState<SearchResult[]>([]);


  const [showMessageSearch, setShowMessageSearch] = useState(false);


  const [highlightedIndices, setHighlightedIndices] = useState<Set<number>>(new Set());


  // Delete confirmation dialog state


  const [deleteDialog, setDeleteDialog] = useState<{ chatId: string; chatName: string; hasChanges: boolean } | null>(null);


  // Toast notification state


  const [toast, setToast] = useState<{ message: string; visible: boolean }>({ message: "", visible: false });


  function showToast(message: string) {


    setToast({ message, visible: true });


    setTimeout(() => setToast({ message: "", visible: false }), 2000);


  }


  setToastCallback(showToast);


  const [showAdvancedSearch, setShowAdvancedSearch] = useState(false);


  const [searchFilters, setSearchFilters] = useState<SearchFilters>({


    query: "",


    date_from: null,


    date_to: null,


    sender: null,


    msg_type: null,


  });


  const [jumpedIndex, setJumpedIndex] = useState<number | null>(null);
  const [followOutput, setFollowOutput] = useState<boolean | 'auto' | 'smooth'>('smooth');

  // Scroll to jumped message when index changes
  useEffect(() => {
    if (jumpedIndex !== null && jumpedIndex >= 0) {
      console.log("Scrolling to jumped index:", jumpedIndex);
      // Small delay to ensure the message list is visible
      setTimeout(() => {
        virtualListRef.current?.scrollToIndex(jumpedIndex);
      }, 100);
    }
  }, [jumpedIndex]);


  const [searchResultCursor, setSearchResultCursor] = useState<number>(0);


  // Link confirmation modal state


  const [linkConfirmUrl, setLinkConfirmUrl] = useState<string | null>(null);


  // Success notification dialog state


  const [successDialog, setSuccessDialog] = useState<{ chatName: string } | null>(null);


  const [deleteSuccessDialog, setDeleteSuccessDialog] = useState<{ chatName: string } | null>(null);


  function openLinkWithConfirm(url: string) {
    setLinkConfirmUrl(url);
  }

  const renderMessageText = createRenderMessageText({ onLinkClick: openLinkWithConfirm });

  function confirmOpenLink() {
    if (linkConfirmUrl) {
      const lower = linkConfirmUrl.toLowerCase();
      if (!lower.startsWith('https://') && !lower.startsWith('http://')) {
        setLinkConfirmUrl(null);
        return;
      }
      openUrl(linkConfirmUrl).catch(console.error);
    }
    setLinkConfirmUrl(null);
  }


  // Profile state


  const [, setMediaBaseDir] = useState<string>("");


  const [profile, setProfile] = useState<Profile | null>(null);


  const [showProfileDialog, setShowProfileDialog] = useState(false);


  const [showGroupDialog, setShowGroupDialog] = useState(false);


  const [editingProfileName, setEditingProfileName] = useState("");


  const [editingProfileNotes, setEditingProfileNotes] = useState("");


  const [editingProfilePhone, setEditingProfilePhone] = useState("");


  const [pendingProfilePhoto, setPendingProfilePhoto] = useState<string | null>(null);


  const [nameHistory, setNameHistory] = useState<NameHistoryEntry[]>([]);


  const [chatBackground, setChatBackground] = useState<string | null>(() => {


    const stored = localStorage.getItem("whatsapp_chat_background");


    return stored || null;


  });


  const [showBackgroundModal, setShowBackgroundModal] = useState(false);


  const [backgroundHistory, setBackgroundHistory] = useState<BackgroundHistoryEntry[]>(() => {


    try { return JSON.parse(localStorage.getItem("whatsapp_background_history") || "[]"); } catch { return []; }


  });


  const [showBgHistory, setShowBgHistory] = useState(false);


  const [defaultBackgrounds, setDefaultBackgrounds] = useState<string[]>([]);


  const [pendingBackground, setPendingBackground] = useState<string | null>(null);


  const [pendingBackgroundSrc, setPendingBackgroundSrc] = useState<string | null>(null);


  const [bgTab, setBgTab] = useState<"default" | "custom">("default");


  // Load username, theme and migrate chats on mount


  useEffect(() => {


    const stored = localStorage.getItem("whatsapp_username");


    if (stored) {


      setUsername(stored);


    } else {


      setShowUsernameDialog(true);


    }


    const storedTheme = localStorage.getItem("whatsapp_theme") as "light" | "dark" | null;


    if (storedTheme) {


      setTheme(storedTheme);


    }


    // Migrate existing JSON chats to SQLite


    invoke("migrate_chats").catch(() => {


      // Migration error silently ignored


    });


    loadChatList();


  }, []);


  // Apply theme class to document


  useEffect(() => {


    document.documentElement.setAttribute("data-theme", theme);


    localStorage.setItem("whatsapp_theme", theme);


  }, [theme]);


  function toggleTheme() {


    setTheme(prev => prev === "light" ? "dark" : "light");


  }


  const handleUsernameSubmit = (name: string) => {


    localStorage.setItem("whatsapp_username", name);


    setUsername(name);


    setShowUsernameDialog(false);


  };


  // Load messages and profile when chat selected


  useEffect(() => {


    // Cancel any in-flight load from the previous chat
    if (loadCancelRef.current) {
      loadCancelRef.current.cancelled = true;
    }

    if (selectedChat) {

      setPendingProfilePhoto(null);


      loadMessages(selectedChat);


      loadProfile(selectedChat);


      invoke<string>("get_media_base_dir", { chatId: selectedChat }).then(setMediaBaseDir);


    }


    // Close message search and media gallery when switching chats


    setShowMediaGallery(false);


    setShowMessageSearch(false);


    setMessageSearchQuery("");


    setMessageSearchResults([]);


    setHighlightedIndices(new Set());


    setJumpedIndex(null);


    setSearchResultCursor(0);


  }, [selectedChat]);


  async function loadProfile(chatId: string) {


    try {


      const data: Profile = await invoke("get_profile", { chatId });

      if (selectedChatRef.current !== chatId) return;

      setProfile(data);


      setEditingProfileName(data.name || "");


      setEditingProfileNotes(data.notes || "");


      setEditingProfilePhone(data.phone_number || "");


    } catch (err) {


      console.error("Failed to load profile:", err);


    }


  }


  async function handlePickCustomBackground() {
    if (isMobile) {
      // On Android, use a hidden <input type="file"> to avoid content URI issues
      const input = document.createElement("input");
      input.type = "file";
      input.accept = "image/*";
      input.onchange = async () => {
        const file = input.files?.[0];
        if (!file) return;
        try {
          const arrayBuffer = await file.arrayBuffer();
          const uint8 = new Uint8Array(arrayBuffer);
          let binary = "";
          const chunkSize = 8192;
          for (let j = 0; j < uint8.length; j += chunkSize) {
            binary += String.fromCharCode(...uint8.subarray(j, j + chunkSize));
          }
          const b64 = btoa(binary);
          const ext = file.name.split(".").pop() ?? "jpg";
          const savedPath = await invoke<string>("save_background_from_b64", { b64, ext });
          // Also create a data URL for preview
          const mimeMap: Record<string, string> = { jpg: "image/jpeg", jpeg: "image/jpeg", png: "image/png", gif: "image/gif", webp: "image/webp", bmp: "image/bmp", svg: "image/svg+xml" };
          const mime = mimeMap[ext.toLowerCase()] ?? "image/jpeg";
          const dataUrl = `data:${mime};base64,${b64}`;
          setPendingBackground(savedPath);
          setPendingBackgroundSrc(dataUrl);
        } catch (e) {
          console.error("Failed to save background:", e);
        }
      };
      input.click();
      return;
    }

    try {

      const path: string | null = await invoke("pick_global_background");

      if (path) {

        setPendingBackground(path);

        // Load custom background as base64 data URL for preview

        try {

          const dataUrl = await invoke<string>("read_file_as_base64", { path });

          setPendingBackgroundSrc(dataUrl);

        } catch (e) {

          console.error("Failed to load background preview:", e);

          setPendingBackgroundSrc(null);

        }

      }

    } catch (err) {

      console.error("Failed to pick background:", err);

    }

  }


  function handleSelectDefaultBackground(filename: string) {


    setPendingBackground(`/app_backgrounds/${filename}`);


    setPendingBackgroundSrc(null);


  }


  function handleRestoreBackground(bgPath: string) {


    setPendingBackground(bgPath);


    setPendingBackgroundSrc(null);


  }


  function handleSaveBackground() {


    // If no selection was made (pendingBackground is null), do nothing


    if (pendingBackground === null) {


      setShowBackgroundModal(false);


      return;


    }


    const backgroundToSave = pendingBackground === "" ? null : pendingBackground;


    setChatBackground(backgroundToSave);


    if (backgroundToSave === null) {


      localStorage.removeItem("whatsapp_chat_background");


      showToast("Background cleared.");


      setPendingBackground(null);


      setPendingBackgroundSrc(null);


      setShowBackgroundModal(false);


      return;


    }


    localStorage.setItem("whatsapp_chat_background", backgroundToSave);


    const isDefaultBackground = backgroundToSave.startsWith("/app_backgrounds/");


    if (!isDefaultBackground) {


      const newEntry: BackgroundHistoryEntry = { id: Date.now(), background_path: backgroundToSave, changed_at: new Date().toISOString() };


      setBackgroundHistory(prev => {


        const existingIndex = prev.findIndex(entry => entry.background_path === backgroundToSave);


        let updated: BackgroundHistoryEntry[];


        if (existingIndex !== -1) {


          updated = [newEntry, ...prev.filter((_, idx) => idx !== existingIndex)];


        } else {


          updated = [newEntry, ...prev];


        }


        localStorage.setItem("whatsapp_background_history", JSON.stringify(updated));


        return updated;


      });


    }


    showToast("Background saved!");


    setPendingBackground(null);


    setPendingBackgroundSrc(null);


    setShowBackgroundModal(false);


  }


  function handleClearBackground() {


    setPendingBackground("");


    setPendingBackgroundSrc(null);


  }


  function handleClearBackgroundHistory() {


    setBackgroundHistory([]);


    localStorage.removeItem("whatsapp_background_history");


    showToast("Background history cleared.");


  }


  function handleDeleteBackgroundEntry(entryId: number) {


    setBackgroundHistory(prev => {


      const updated = prev.filter(entry => entry.id !== entryId);


      localStorage.setItem("whatsapp_background_history", JSON.stringify(updated));


      return updated;


    });


    showToast("Background removed from history.");


  }


  async function loadNameHistory(chatId: string) {


    try {


      const history: NameHistoryEntry[] = await invoke("get_name_history", { chatId });


      setNameHistory(history);


    } catch (err) {


      console.error("Failed to load name history:", err);


    }


  }


  async function handleRestoreName(name: string) {


    if (!selectedChat) return;


    await invoke("revert_profile_name", { chatId: selectedChat, name });


    setEditingProfileName(name);


    setProfile(prev => prev ? { ...prev, name } : null);


    showToast(`Name restored to "${name}"`);


    await loadNameHistory(selectedChat);


    loadChatList();


  }


  async function handleResetName() {


    if (!selectedChat) return;


    await invoke("revert_profile_name", { chatId: selectedChat, name: null });


    setProfile(prev => prev ? { ...prev, name: null } : null);


    showToast("Name reset to original");


    await loadNameHistory(selectedChat);


    // Fetch fresh chat list so we get the COALESCE'd original name


    const freshChats: ChatMeta[] = await invoke("get_chat_list");


    setChats(freshChats);


    const refreshed = freshChats.find((c: ChatMeta) => c.id === selectedChat);


    setEditingProfileName(refreshed?.name ?? "");


  }


  async function saveProfile() {


    if (!selectedChat) return;


    try {


      if (pendingProfilePhoto === "") {


        await invoke("remove_profile_photo", { chatId: selectedChat });


      }


      await invoke("update_profile", {


        chatId: selectedChat,


        name: editingProfileName || null,


        notes: editingProfileNotes || null,


        photoPath: pendingProfilePhoto && pendingProfilePhoto !== "" ? pendingProfilePhoto : null,


        phoneNumber: editingProfilePhone || null


      });


      const newPhotoPath = pendingProfilePhoto === "" ? null : (pendingProfilePhoto ?? profile?.photo_path ?? null);


      setProfile(prev => prev ? { ...prev, name: editingProfileName, notes: editingProfileNotes, phone_number: editingProfilePhone, photo_path: newPhotoPath } : null);


      setChats(prev => prev.map(c => c.id === selectedChat ? { ...c, photo_path: newPhotoPath } : c));


      setPendingProfilePhoto(null);


      showToast("Profile saved successfully!");


      // Refresh chat list to show updated name


      loadChatList();


    } catch (err) {


      console.error("Failed to save profile:", err);


      showToast("Failed to save profile: " + err);


    }


  }


  async function handleProfilePhotoUpload() {


    try {


      const path: string | null = await invoke("pick_profile_photo");


      if (path) {


        setPendingProfilePhoto(path);


      }


    } catch (err) {


      console.error("Failed to upload photo:", err);


    }


  }



  function handlePhotoRemove() {


    setPendingProfilePhoto(""); // Empty string indicates photo removal


  }


  // Auto-scroll to bottom only on fresh loads, and re-anchor while images are loading


  const isInitialLoad = useRef(false);


  useEffect(() => {


    if (shouldScrollToBottom.current) {


      shouldScrollToBottom.current = false;


      isInitialLoad.current = true;


      messagesEndRef.current?.scrollIntoView({ behavior: "instant" });


    }


  }, [messages]);


  // Re-anchor to bottom as images/media finish loading after initial chat open


  useEffect(() => {


    const container = messagesContainerRef.current;


    if (!container) return;


    const snapToBottom = () => {


      if (isInitialLoad.current) {


        container.scrollTop = container.scrollHeight;


      }


    };


    // Attach load listeners to all existing images


    const attachToImages = () => {


      container.querySelectorAll("img, video").forEach((el) => {


        el.addEventListener("load", snapToBottom, { once: true });


        el.addEventListener("loadedmetadata", snapToBottom, { once: true });


      });


    };


    attachToImages();


    // Also watch for dynamically added images (lazy-rendered)


    const mutation = new MutationObserver(() => {


      attachToImages();


      snapToBottom();


    });


    mutation.observe(container, { childList: true, subtree: true });


    const onScroll = () => {


      const { scrollTop, scrollHeight, clientHeight } = container;


      if (scrollTop + clientHeight < scrollHeight - 60) {


        isInitialLoad.current = false;


      }


    };


    container.addEventListener("scroll", onScroll, { passive: true });


    return () => {


      mutation.disconnect();


      container.removeEventListener("scroll", onScroll);


    };


  }, [messages]);


  // Scroll detection for chat messages


  useEffect(() => {


    const container = messagesContainerRef.current;


    if (!container) return;


    const updateScrollState = () => {


      const { scrollTop, scrollHeight, clientHeight } = container;


      const atTop = scrollTop <= 10;


      const atBottom = scrollTop + clientHeight >= scrollHeight - 10;


      const hasOverflow = scrollHeight > clientHeight;


      setChatScrollState({ atTop, atBottom, hasOverflow });


    };


    let rafId: number | null = null;


    const throttledUpdate = () => {


      if (rafId !== null) return;


      rafId = requestAnimationFrame(() => {


        rafId = null;


        updateScrollState();


      });


    };


    updateScrollState();


    container.addEventListener('scroll', throttledUpdate, { passive: true });


    window.addEventListener('resize', updateScrollState);


    return () => {


      if (rafId !== null) cancelAnimationFrame(rafId);


      container.removeEventListener('scroll', throttledUpdate);


      window.removeEventListener('resize', updateScrollState);


    };


  }, [messages, selectedChat]);


  // Jump functions


  const virtualListRef = useRef<VirtualMessageListRef>(null);
  const searchOverlayRef = useRef<HTMLDivElement>(null);


  const scrollToChatTop = () => {


    virtualListRef.current?.scrollToTop();


  };


  const scrollToChatBottom = () => {


    virtualListRef.current?.scrollToBottom();


  };


  async function loadChatList() {


    try {


      console.log("[LOAD_CHAT_LIST] Loading chat list...");


      const list: ChatMeta[] = await invoke("get_chat_list");


      console.log("[LOAD_CHAT_LIST] Received", list.length, "chats");


      // Force new array reference to ensure React detects change


      setChats([...list]);


      console.log("[LOAD_CHAT_LIST] State updated");


    } catch (err) {


      console.error("Failed to load chats:", err);


    }


  }


  async function refreshChatSearch() {


    if (chatSearchQuery.trim()) {


      console.log("[REFRESH] Refreshing chat search for query:", chatSearchQuery);


      try {


        const results: ChatMeta[] = await invoke("search_chats", { query: chatSearchQuery });


        setChatSearchResults(results);


        console.log("[REFRESH] Chat search refreshed, found", results.length, "results");


      } catch (err) {


        console.error("Chat search refresh failed:", err);


      }


    } else {


      console.log("[REFRESH] No chat search query active, skipping search refresh");


    }


  }


  async function loadMessages(chatId: string) {


    console.log("[LOAD_MESSAGES] Starting to load messages for chat:", chatId);

    // Create a new cancel token for this load; cancel any previous
    const token = { cancelled: false };
    loadCancelRef.current = token;

    setLoading(true);


    try {


      // Get total count first to calculate offset for last 100 messages


      const totalCount: number = await invoke("get_chat_message_count", { chatId });


      console.log("[LOAD_MESSAGES] Total message count:", totalCount);


      // Load the LAST 100 messages (most recent) for immediate render


      const offset = totalCount > 100 ? totalCount - 100 : 0;


      const initialData: ChatData = await invoke("get_chat_messages", { 


        chatId, 


        limit: 100, 


        offset 


      });


      console.log("[LOAD_MESSAGES] Received", initialData.messages.length, "messages");


      if (token.cancelled) return;

      shouldScrollToBottom.current = true;


      // Set last 100 messages (most recent) immediately for fast render - force new array


      setMessages([...initialData.messages]);


      console.log("[LOAD_MESSAGES] Messages state updated");


      // Stream earlier messages in chunks to avoid UI stutter


      if (totalCount > 100) {


        const remainingCount = totalCount - 100;


        const chunkSize = 100;


        const chunks = Math.ceil(remainingCount / chunkSize);


        // Load chunks progressively using requestIdleCallback or setTimeout


        for (let i = 0; i < chunks; i++) {


          if (token.cancelled) break;


          await new Promise<void>(resolve => {


            const loadChunk = () => {


              if (token.cancelled) { resolve(); return; }


              const offset = i * chunkSize;


              const limit = Math.min(chunkSize, remainingCount - offset);


              invoke<ChatData>("get_chat_messages", { 


                chatId, 


                limit, 


                offset 


              }).then(chunkData => {


                if (!token.cancelled && chunkData.messages.length > 0) {


                  // Insert chunk in correct position (before existing messages)


                  setMessages(prev => {


                    const before = prev.slice(0, offset);


                    const after = prev.slice(offset);


                    return [...before, ...chunkData.messages, ...after];


                  });


                }


                resolve();


              });


            };


            // Use requestIdleCallback if available, else setTimeout


            if (typeof window !== 'undefined' && 'requestIdleCallback' in window) {


              window.requestIdleCallback(loadChunk, { timeout: 100 });


            } else {


              setTimeout(loadChunk, 0);


            }


          });


        }


      }


    } catch (err) {


      console.error("Failed to load messages:", err);


    } finally {


      setLoading(false);


    }


  }


  async function handleImport() {
    // On Android, tauri-plugin-dialog file picker does not work. Use a hidden
    // <input type="file"> and send raw bytes to Rust via import_zip_from_bytes.
    if (/android/i.test(navigator.userAgent)) {
      const input = document.createElement("input");
      input.type = "file";
      input.accept = ".zip,application/zip,application/octet-stream";
      input.multiple = true;
      input.onchange = async () => {
        const files = Array.from(input.files ?? []);
        if (files.length === 0) return;
        setImporting(true);
        setImportProgress({ current: 0, total: files.length });
        setImportDetail(null);
        const importedIds: string[] = [];
        try {
          for (let i = 0; i < files.length; i++) {
            setImportProgress({ current: i + 1, total: files.length });
            const arrayBuffer = await files[i].arrayBuffer();
            const uint8 = new Uint8Array(arrayBuffer);
            let binary = "";
            const chunkSize = 8192;
            for (let j = 0; j < uint8.length; j += chunkSize) {
              binary += String.fromCharCode(...uint8.subarray(j, j + chunkSize));
            }
            const b64 = btoa(binary);
            const chatIds: string[] = await invoke("import_zip_from_bytes", { b64, filename: files[i].name });
            importedIds.push(...chatIds);
          }
          // Use ref to get current selectedChat (avoids stale closure)
          const currentChat = selectedChatRef.current;
          console.log("[IMPORT] Refreshing after import, currentChat:", currentChat);
          
          // Delay to ensure backend has finished writing
          await new Promise(resolve => setTimeout(resolve, 300));
          
          await loadChatList();
          await refreshChatSearch();
          setShowStartScreen(false);
          if (currentChat) {
            // Always refresh currently open chat in case it was merged/updated
            console.log("[IMPORT] Reloading messages and profile for chat:", currentChat);
            await loadMessages(currentChat);
            await loadProfile(currentChat);
            invoke<string>("get_media_base_dir", { chatId: currentChat }).then(setMediaBaseDir);
            console.log("[IMPORT] Chat refresh complete");
          } else if (importedIds.length > 0) {
            // No chat open, select the last imported one
            const targetId = importedIds[importedIds.length - 1];
            setSelectedChat(targetId);
            setSelectedMessageIndices(new Set());
            setMultiSelectMode(false);
            setLastSelectedIndex(null);
            setLastLongPressedIndex(null);
          }
          showToast(`Imported ${importedIds.length} chat(s)`);
        } catch (err) {
          showToast("Import failed: " + err);
        } finally {
          setImporting(false);
          setImportProgress(null);
          setImportDetail(null);
        }
      };
      input.click();
      return;
    }


    setImporting(true);


    setImportProgress(null);


    setImportDetail(null);


    try {


      // Desktop: use the native file dialog
      const paths: string[] = await invoke("pick_zip_files");


      if (paths.length === 0) { setImporting(false); return; }


      setImportProgress({ current: 0, total: paths.length });


      const importedIds: string[] = [];


      for (let i = 0; i < paths.length; i++) {


        setImportProgress({ current: i + 1, total: paths.length });


        // import_from_export auto-detects export ZIPs (with meta.json) vs regular WhatsApp ZIPs


        const chatIds: string[] = await invoke("import_from_export", { zipPath: paths[i] });


        // Apply saved modifications for each imported chat


        for (const chatId of chatIds) {


          const chatList: ChatMeta[] = await invoke("get_chat_list");


          const thisChat = chatList.find(c => c.id === chatId);


          const chatName = thisChat?.name || chatId;


          const saveKey = `chat_modifications_${chatName}`;


          const savedModifications = localStorage.getItem(saveKey);


          if (savedModifications) {


            try {


              await invoke("apply_chat_modifications", {


                chatId,


                modificationsJson: savedModifications


              });


              await invoke("clear_modification_flags", { chatId });


            } catch (err) {


              console.error(`[IMPORT] Failed to apply modifications:`, err);


            }


          }


          importedIds.push(chatId);


        }


      }


      console.log("[DESKTOP IMPORT] Refreshing after import, selectedChat:", selectedChat);
      
      // Small delay to ensure backend has finished writing
      await new Promise(resolve => setTimeout(resolve, 100));
      
      await loadChatList();
      await refreshChatSearch();
      setShowStartScreen(false);

      if (selectedChat) {
        // Always refresh currently open chat in case it was merged/updated
        console.log("[DESKTOP IMPORT] Reloading messages and profile for chat:", selectedChat);
        await loadMessages(selectedChat);
        await loadProfile(selectedChat);
        invoke<string>("get_media_base_dir", { chatId: selectedChat }).then(setMediaBaseDir);
        console.log("[DESKTOP IMPORT] Chat refresh complete");
      } else if (importedIds.length === 1) {
        // No chat open, select the imported one if only one was imported
        setSelectedChat(importedIds[0]);
        setSelectedMessageIndices(new Set());
        setMultiSelectMode(false);
        setLastSelectedIndex(null);
        setLastLongPressedIndex(null);
      }


    } catch (err) {


      showToast("Import failed: " + err);


    } finally {


      setImporting(false);


      setImportProgress(null);


      setImportDetail(null);


    }


  }


  // Called when app is opened via WhatsApp "Export Chat" share intent on Android
  async function handleShareImport(zipPath: string) {
    if (importing) return;
    console.log("[SHARE IMPORT] Starting import from:", zipPath);
    showToast("Importing chat from WhatsApp...");
    setImporting(true);
    setImportProgress({ current: 0, total: 1 });
    setImportDetail(null);
    try {
      const chatIds: string[] = await invoke("import_from_export", { zipPath });
      // Use ref to get current selectedChat (avoids stale closure)
      const currentChat = selectedChatRef.current;
      console.log("[SHARE IMPORT] Refreshing after import, currentChat:", currentChat);
      
      // Delay to ensure backend has finished writing
      await new Promise(resolve => setTimeout(resolve, 300));
      
      await loadChatList();
      await refreshChatSearch();
      setShowStartScreen(false);
      if (currentChat) {
        // Always refresh currently open chat in case it was merged/updated
        console.log("[SHARE IMPORT] Reloading messages and profile for chat:", currentChat);
        await loadMessages(currentChat);
        await loadProfile(currentChat);
        invoke<string>("get_media_base_dir", { chatId: currentChat }).then(setMediaBaseDir);
        console.log("[SHARE IMPORT] Chat refresh complete");
      } else if (chatIds.length > 0) {
        // No chat open, select the last imported one
        const targetId = chatIds[chatIds.length - 1];
        setSelectedChat(targetId);
        setSelectedMessageIndices(new Set());
        setMultiSelectMode(false);
        setLastSelectedIndex(null);
        setLastLongPressedIndex(null);
      }
      setImportProgress({ current: 1, total: 1 });
    } catch (e) {
      console.error("Share import failed:", e);
    } finally {
      setImporting(false);
      setImportProgress(null);
      setImportDetail(null);
    }
  }

  // On Android: poll for share intents more aggressively (every 1 second)
  useEffect(() => {
    if (!isAndroid) return;
    
    let cancelled = false;
    const checkForShare = async () => {
      if (cancelled) return;
      try {
        const zipPath = await invoke<string | null>("get_pending_share");
        if (zipPath) {
          showToast("Found shared chat, importing...");
          await handleShareImport(zipPath);
        }
      } catch (e) {
        // Silently ignore errors during polling
      }
    };

    // Check immediately on mount
    checkForShare();

    // Poll every 1 second for faster detection
    const interval = setInterval(checkForShare, 1000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  // Poll import progress from backend while importing


  useEffect(() => {


    if (!importing) return;


    const interval = setInterval(async () => {


      try {


        const p = await invoke<{ messages: number; media: number; phase: string }>("get_import_progress");


        setImportDetail(p);


      } catch {}


    }, 300);


    return () => clearInterval(interval);


  }, [importing]);


  // --- Export functions ---


  const [exporting, setExporting] = useState(false);


  const [showExportMenu, setShowExportMenu] = useState(false);


  const [exportSuccessMsg, setExportSuccessMsg] = useState<string | null>(null);


  async function handleExportChat(chatId: string) {


    setExporting(true);


    setShowExportMenu(false);


    try {


      const result: string = await invoke("export_chat_zip", { chatId });


      if (result !== "cancelled") {


        setExportSuccessMsg("Chat has been saved to your chosen location.");


      }


    } catch (err) {


      showToast("Export failed: " + err);


    } finally {


      setExporting(false);


    }


  }


  async function handleExportAll() {


    setExporting(true);


    setShowExportMenu(false);


    try {


      const result: string = await invoke("export_all_chats_zip");


      if (result !== "cancelled") {


        setExportSuccessMsg(`All ${chats.length} chats have been saved to your chosen location.`);


      }


    } catch (err) {


      showToast("Export failed: " + err);


    } finally {


      setExporting(false);


    }


  }


  // --- Clear All Chats ---


  const [showClearAllConfirm, setShowClearAllConfirm] = useState(false);


  const [clearAllHasChanges, setClearAllHasChanges] = useState(false);


  async function openClearAllConfirm() {


    // Check if any chat has unsaved modifications


    let anyChanges = false;


    for (const chat of chats) {


      try {


        const mods: string = await invoke("export_chat_modifications", { chatId: chat.id });


        if (mods !== "[]") { anyChanges = true; break; }


      } catch { /* ignore */ }


    }


    setClearAllHasChanges(anyChanges);


    setShowClearAllConfirm(true);


  }


  async function handleClearAllSaveAndDelete() {


    try {


      // Save pending profile changes for the currently selected chat first
      if (selectedChat && (editingProfileName !== profile?.name || editingProfileNotes !== profile?.notes || editingProfilePhone !== profile?.phone_number || pendingProfilePhoto !== null)) {
        await saveProfile();
      }

      // Save modifications for every chat to localStorage before deleting


      for (const chat of chats) {


        try {


          const modifications: string = await invoke("export_chat_modifications", { chatId: chat.id });


          if (modifications === "[]") continue;


          const saveKey = `chat_modifications_${chat.name}`;


          const existingMods = localStorage.getItem(saveKey);


          let allMods: any[] = JSON.parse(modifications);


          if (existingMods) {


            try {


              const existing = JSON.parse(existingMods);


              const modMap = new Map();


              for (const mod of existing) modMap.set(mod.type === "message" ? mod.message_index : "profile", mod);


              for (const mod of allMods) modMap.set(mod.type === "message" ? mod.message_index : "profile", mod);


              allMods = Array.from(modMap.values());


            } catch { /* use new mods */ }


          }


          localStorage.setItem(saveKey, JSON.stringify(allMods));


        } catch { /* skip chat */ }


      }


      const count: number = await invoke("clear_all_chats");


      setSelectedChat(null);


      await loadChatList();


      showToast(`Saved history & deleted ${count} chat${count !== 1 ? 's' : ''}`);


    } catch (err) {


      showToast("Clear failed: " + err);


    } finally {


      setShowClearAllConfirm(false);


    }


  }


  async function handleClearAllChats() {


    try {


      const count: number = await invoke("clear_all_chats");


      setSelectedChat(null);


      await loadChatList();


      showToast(`Deleted ${count} chat${count !== 1 ? 's' : ''}`);


    } catch (err) {


      showToast("Clear failed: " + err);


    } finally {


      setShowClearAllConfirm(false);


    }


  }


  async function handleDelete(chatId: string, e: React.MouseEvent) {


    e.stopPropagation();


    // Find the chat name for the dialog


    const chat = chats.find(c => c.id === chatId);


    const chatName = chat?.name || "Unknown Chat";


    try {


      // Check if there are any modifications for this chat


      const modifications: string = await invoke("export_chat_modifications", { chatId });


      const hasChanges = modifications !== "[]";


      setDeleteDialog({ chatId, chatName, hasChanges });


    } catch (err) {


      console.error("Failed to check modifications:", err);


      // If we can't check, assume no changes


      setDeleteDialog({ chatId, chatName, hasChanges: false });


    }


  }


  async function handleSaveChanges() {


    if (!deleteDialog) return;


    try {


      // Export chat modifications


      const modifications: string = await invoke("export_chat_modifications", { 


        chatId: deleteDialog.chatId 


      });


      if (modifications === "[]") {


        showToast("No changes to save for this chat.");


        setDeleteDialog(null);


        return;


      }


      // Save modifications to localStorage for later re-application


      // Use chat name as stable key (chatId is random on each import)


      const saveKey = `chat_modifications_${deleteDialog.chatName}`;


      // Merge with existing modifications (don't overwrite)


      const existingMods = localStorage.getItem(saveKey);


      let allModifications: any[] = JSON.parse(modifications);


      if (existingMods) {


        try {


          const existing = JSON.parse(existingMods);


          // Create a map by message_index to deduplicate


          const modMap = new Map();


          // Add existing first


          for (const mod of existing) {


            if (mod.type === "message") {


              modMap.set(mod.message_index, mod);


            } else if (mod.type === "profile") {


              modMap.set("profile", mod);


            }


          }


          // Add/override with new ones


          for (const mod of allModifications) {


            if (mod.type === "message") {


              modMap.set(mod.message_index, mod);


            } else if (mod.type === "profile") {


              modMap.set("profile", mod);


            }


          }


          allModifications = Array.from(modMap.values());


        } catch (e) {


          // Parse error, use new modifications


        }


      }


      const mergedJson = JSON.stringify(allModifications);


      localStorage.setItem(saveKey, mergedJson);


      // Now delete the chat after saving changes


      await invoke("delete_chat", { chatId: deleteDialog.chatId });


      if (selectedChat === deleteDialog.chatId) {


        setSelectedChat(null);


      }


      setChats(prev => prev.filter(chat => chat.id !== deleteDialog.chatId));


      setSuccessDialog({ chatName: deleteDialog.chatName });


      setDeleteDialog(null);


    } catch (err) {


      console.error("Failed to save changes:", err);


      showToast("Failed to save changes: " + err);


    }


  }


  async function handleDeleteEverything() {


    if (!deleteDialog) return;


    try {


      await invoke("delete_chat", { chatId: deleteDialog.chatId });


      // Clear any saved modifications from localStorage


      const saveKey = `chat_modifications_${deleteDialog.chatName}`;


      localStorage.removeItem(saveKey);


      if (selectedChat === deleteDialog.chatId) {


        setSelectedChat(null);


      }


      setChats(prev => prev.filter(chat => chat.id !== deleteDialog.chatId));


      setDeleteDialog(null);


      // Show delete success dialog


      setDeleteSuccessDialog({ chatName: deleteDialog.chatName });


    } catch (err) {


      showToast("Failed to delete chat");


    }


  }


  function handleCancelDelete() {


    setDeleteDialog(null);


  }


  // Chat search function


  async function handleChatSearch(e: React.ChangeEvent<HTMLInputElement>) {


    const query = e.target.value;


    setChatSearchQuery(query);


    if (query.trim()) {


      setIsSearchingChats(true);


      try {


        const results: ChatMeta[] = await invoke("search_chats", { query });


        setChatSearchResults(results);


      } catch (err) {


        console.error("Chat search failed:", err);


      }


    } else {


      setIsSearchingChats(false);


      setChatSearchResults([]);


      loadChatList();


    }


  }


  // Message search functions


  async function handleMessageSearch() {


    if (!selectedChat || !messageSearchQuery.trim()) return;


    try {


      const results: SearchResult[] = await invoke("search_messages", {


        chatId: selectedChat,


        query: messageSearchQuery


      });


      // Also match against the human-readable date label (Today / Yesterday / "30 April 2024")


      // so queries like "april", "today", "yesterday" work even if the raw timestamp doesn't contain them


      const q = messageSearchQuery.toLowerCase();


      const seenIndices = new Set(results.map(r => r.message_index));


      messages.forEach((msg, idx) => {


        if (seenIndices.has(idx)) return;


        if (formatDate(msg.timestamp).toLowerCase().includes(q)) {


          seenIndices.add(idx);


          results.push({


            message_index: idx,


            timestamp: msg.timestamp,


            sender: msg.sender,


            content: msg.content,


            msg_type: msg.type,


          });


        }


      });


      // Re-sort by message index to keep chronological order


      results.sort((a, b) => a.message_index - b.message_index);


      setMessageSearchResults(results);


      setHighlightedIndices(new Set(results.map(r => r.message_index)));


      setSearchResultCursor(0);


      // Auto-jump to first result


      if (results.length > 0) {


        scrollToResult(results[0].message_index);


        setJumpedIndex(results[0].message_index);


      }


    } catch (err) {


      console.error("Message search failed:", err);


    }


  }


  function handleMessageSearchInput(e: React.ChangeEvent<HTMLInputElement>) {


    setMessageSearchQuery(e.target.value);


    // Clear stale results any time the query changes


    setMessageSearchResults([]);


    setHighlightedIndices(new Set());


    setJumpedIndex(null);


    setSearchResultCursor(0);


  }


  function scrollToResult(msgIndex: number) {


    virtualListRef.current?.scrollToIndex(msgIndex);


  }


  function jumpToResult(cursor: number) {


    if (messageSearchResults.length === 0) return;


    const clamped = (cursor + messageSearchResults.length) % messageSearchResults.length;


    setSearchResultCursor(clamped);


    const msgIndex = messageSearchResults[clamped].message_index;


    setJumpedIndex(msgIndex);


    scrollToResult(msgIndex);


  }


  async function handleAdvancedSearch() {


    if (!selectedChat) return;


    try {


      const results: SearchResult[] = await invoke("search_messages_filtered", {


        chatId: selectedChat,


        filters: searchFilters


      });


      // Also match against the human-readable date label for date-based queries


      const q = searchFilters.query.toLowerCase();


      const seenIndices = new Set(results.map(r => r.message_index));


      if (q) {


        messages.forEach((msg, idx) => {


          if (seenIndices.has(idx)) return;


          if (formatDate(msg.timestamp).toLowerCase().includes(q)) {


            seenIndices.add(idx);


            results.push({


              message_index: idx,


              timestamp: msg.timestamp,


              sender: msg.sender,


              content: msg.content,


              msg_type: msg.type,


            });


          }


        });


      }


      // Re-sort by message index to keep chronological order


      results.sort((a, b) => a.message_index - b.message_index);


      setMessageSearchResults(results);


      setHighlightedIndices(new Set(results.map(r => r.message_index)));


      setSearchResultCursor(0);


      // Auto-jump to first result


      if (results.length > 0) {


        scrollToResult(results[0].message_index);


        setJumpedIndex(results[0].message_index);


      }


    } catch (err) {


      console.error("Advanced search failed:", err);


    }


  }


  function jumpToMessage(index: number) {


    const pos = messageSearchResults.findIndex(r => r.message_index === index);


    if (pos !== -1) setSearchResultCursor(pos);


    console.log("jumpToMessage called with index:", index);


    setJumpedIndex(index);


    setShowFavorites(false);


    // Disable followOutput to prevent auto-scroll to bottom
    setFollowOutput(false);


    // Re-enable followOutput after a short delay
    setTimeout(() => setFollowOutput('smooth'), 500);


  }


function MessageRenderer({


  msg,


  idx,


  username,


  selectedChatData,


  selectedChat,


  messageRefs,


  selectedMessageIndices,


  setSelectedMessageIndices,


  highlightedIndices,


  jumpedIndex,


  openChatLightbox,


  disableInteractions = false,


  hideExpandButton = false,


  onTagSaved,


  showToast


}: {


  msg: Message;


  idx: number;


  username: string;


  selectedChatData: any;


  selectedChat: string;


  messageRefs: React.MutableRefObject<(HTMLElement | null)[]>;


  selectedMessageIndices: Set<number>;


  setSelectedMessageIndices: React.Dispatch<React.SetStateAction<Set<number>>>;


  highlightedIndices: Set<number>;


  jumpedIndex: number | null;


  openChatLightbox: (data: any) => void;


  disableInteractions?: boolean;


  hideExpandButton?: boolean;


  onTagSaved?: (patch: Partial<Message>) => void;


  showToast?: (message: string) => void;


}) {


  const isMe = msg.sender === username;


  // Media type detection


  const isFileMsg = msg.type === "file" && msg.media;


  const tagExtTrimmed = (msg.tag_ext ?? "").trim();


  const effExt = isFileMsg ? (tagExtTrimmed || msg.media?.split(".").pop() || "").toLowerCase().replace(/^\./, "") : "";


  const isImageFile = isFileMsg && IMAGE_EXTS.has(effExt);


  const isVideoFile = isFileMsg && VIDEO_EXTS.has(effExt);


  const isAudioFile = isFileMsg && AUDIO_EXTS.has(effExt);


  return (


    <div


      ref={(el) => { messageRefs.current[idx] = el; }}


      className={`message ${isMe ? "sent" : "received"}${msg.type === "sticker" ? " sticker-message" : ""}${highlightedIndices.has(idx) ? " search-match" : ""}${jumpedIndex === idx ? " jumped" : ""}${selectedMessageIndices.has(idx) ? " selected" : ""}${disableInteractions ? " favorites-message" : ""}`}


      onClick={disableInteractions ? undefined : (e) => {


        if (!e.defaultPrevented) {


          const newSelected = new Set(selectedMessageIndices);


          if (newSelected.has(idx)) {


            newSelected.delete(idx);


          } else {


            newSelected.add(idx);


          }


          setSelectedMessageIndices(newSelected);


        }


      }}


    >


      {selectedChatData?.is_group && (


        <span className="sender-name">{msg.sender}{isMe ? " (You)" : ""}</span>


      )}


      {msg.type === "image" && msg.media && (


        <div className="media-content image" onClick={disableInteractions ? undefined : (e) => {


          if (selectedMessageIndices.size > 0) {


            e.preventDefault();


            e.stopPropagation();


            const newSelected = new Set(selectedMessageIndices);


            if (newSelected.has(idx)) {


              newSelected.delete(idx);


            } else {


              newSelected.add(idx);


            }


            setSelectedMessageIndices(newSelected);


          } else {


            e.preventDefault();


            e.stopPropagation();


            openChatLightbox({ ...msg, _idx: idx });


          }


        }} style={{ cursor: disableInteractions ? "default" : "pointer" }}>


          <LazyMediaImage


            chatId={selectedChat}


            index={idx}


            filename={msg.media}


            onLoad={(e) => {


              (e.target as HTMLImageElement).parentElement?.classList.add("loaded");


            }}


          />


        </div>


      )}


      {msg.type === "sticker" && msg.media && (


        <div className="media-content sticker">


          <StickerImage


            chatId={selectedChat}


            filename={msg.media}


          />


        </div>


      )}


      {msg.type === "video" && msg.media && selectedChat && (


        <div className="media-content video" onClick={disableInteractions ? undefined : (e) => {


          if (selectedMessageIndices.size > 0) {


            e.preventDefault();


            e.stopPropagation();


            const newSelected = new Set(selectedMessageIndices);


            if (newSelected.has(idx)) {


              newSelected.delete(idx);


            } else {


              newSelected.add(idx);


            }


            setSelectedMessageIndices(newSelected);


          } else {


            e.preventDefault();


            e.stopPropagation();


            openChatLightbox({ ...msg, _idx: idx });


          }


        }} style={{ cursor: disableInteractions ? "default" : "pointer" }}>


          <VideoPlayer


            chatId={selectedChat}


            filename={msg.media}


            controls


            className="message-video"


          />


        </div>


      )}


      {msg.type === "gif" && msg.media && selectedChat && (


        <div className="media-content gif" onClick={disableInteractions ? undefined : (e) => {


          if (selectedMessageIndices.size > 0) {


            e.preventDefault();


            e.stopPropagation();


            const newSelected = new Set(selectedMessageIndices);


            if (newSelected.has(idx)) {


              newSelected.delete(idx);


            } else {


              newSelected.add(idx);


            }


            setSelectedMessageIndices(newSelected);


          } else {


            e.preventDefault();


            e.stopPropagation();


            openChatLightbox({ ...msg, _idx: idx });


          }


        }} style={{ cursor: disableInteractions ? "default" : "pointer" }}>


          <GifPlayer


            chatId={selectedChat}


            filename={msg.media}


          />


        </div>


      )}


      {msg.type === "audio" && msg.media && (


        <div className="media-content audio">


          <AudioBase64 chatId={selectedChat} filename={msg.media} />


          {msg.duration && <span className="audio-duration">{msg.duration}</span>}


        </div>


      )}


      {msg.type === "location" && msg.media && (


        <div className="media-content location-attachment">


          <span className="location-icon">📍</span>


          <div className="location-info">


            <span className="location-label">Location</span>


            <span className="location-url">{msg.media}</span>


          </div>


          <button


            className="file-open-btn"


            title="Open in maps"


            onClick={() => openLinkWithConfirm(msg.media!)}


          >


            Open Maps


          </button>


        </div>


      )}


      {/* Media display using precomputed detection variables */}


      {isFileMsg && !isImageFile && !isVideoFile && !isAudioFile && (


        <FileAttachment


          msg={msg}


          idx={idx}


          chatId={selectedChat}


          onTagSaved={onTagSaved ?? (() => {})}


          showToast={showToast ?? (() => {})}


        />


      )}


      {msg.content && !msg.content.includes("(file attached)") && !msg.content.includes("(bestand bijgevoegd)") && msg.type !== "location" && (


        <div className="message-text">


          <ExpandableText content={msg.content} renderFn={renderMessageText} hideExpandButton={hideExpandButton} />


        </div>


      )}


      <span className="message-time">


        {formatTime(msg.timestamp)}


        {msg.is_favorite && <span className="message-star-indicator">⭐</span>}


      </span>


    </div>


  );


}


function FavoritesGallery({


  messages,


  chatId,


  onClose,


  onJumpToMessage,


  username,


  selectedChatData,


  selectedMessageIndices,


  setSelectedMessageIndices,


  highlightedIndices,


  jumpedIndex,


  openChatLightbox


}: {


  messages: Message[];


  chatId: string;


  onClose: () => void;


  onJumpToMessage: (index: number) => void;


  username: string;


  selectedChatData: any;


  selectedMessageIndices: Set<number>;


  setSelectedMessageIndices: React.Dispatch<React.SetStateAction<Set<number>>>;


  highlightedIndices: Set<number>;


  jumpedIndex: number | null;


  openChatLightbox: (data: any) => void;


}) {


  const messageRefs = useRef<(HTMLElement | null)[]>([]);


  return (


    <div className="favorites-gallery">


      <div className="favorites-header">


        <h3>Starred Messages</h3>


        <button className="favorites-close" onClick={onClose}>✕</button>


      </div>


      <div className="favorites-list">


        {messages.length === 0 ? (


          <div className="favorites-empty">No starred messages yet</div>


        ) : (


          messages.map((msg, index) => (


            <div key={index} onClick={() => { onClose(); onJumpToMessage((msg as any)._idx); }} style={{ cursor: "pointer" }}>


              <MessageRenderer


                msg={msg}


                idx={index}


                username={username}


                selectedChatData={selectedChatData}


                selectedChat={chatId}


                messageRefs={messageRefs}


                selectedMessageIndices={selectedMessageIndices}


                setSelectedMessageIndices={setSelectedMessageIndices}


                highlightedIndices={highlightedIndices}


                jumpedIndex={jumpedIndex}


                openChatLightbox={openChatLightbox}


                disableInteractions={true}


                hideExpandButton={true}


              />


            </div>


          ))


        )}


      </div>


    </div>


  );


}


  // Chat media lightbox helpers


  const chatMediaMessages = messages.map((m, i) => ({ ...m, _idx: i })).filter(m => {


    if (!m.media) return false;


    if (m.type === "image" || m.type === "video" || m.type === "gif") return true;


    // Include file messages that are images/videos based on extension/tag


    if (m.type === "file") {


      const tagExt = (m.tag_ext ?? "").trim();


      const ext = (tagExt || m.media.split(".").pop() || "").toLowerCase();


      return IMAGE_EXTS.has(ext) || VIDEO_EXTS.has(ext);


    }


  });


  function openChatLightbox(msg: Message & { _idx: number; hideControls?: boolean }) {


    const mediaIndex = chatMediaMessages.findIndex(m => m._idx === msg._idx);


    // Detect media type from extension/tag for file messages


    let lightboxType: "image" | "video" | "gif" = msg.type as "image" | "video" | "gif";


    if (msg.type === "file" && msg.media) {


      const tagExt = (msg.tag_ext ?? "").trim();


      const ext = (tagExt || msg.media.split(".").pop() || "").toLowerCase();


      if (IMAGE_EXTS.has(ext)) lightboxType = "image";


      else if (VIDEO_EXTS.has(ext)) lightboxType = "video";


      else if (AUDIO_EXTS.has(ext)) lightboxType = "video"; // Audio plays in video player


    }


    setChatLightbox({ filename: msg.media!, type: lightboxType, sender: msg.sender, timestamp: msg.timestamp, index: mediaIndex >= 0 ? mediaIndex : 0, hideControls: msg.hideControls });


  }


function goToChatPrev() {


  if (!chatLightbox || chatMediaMessages.length <= 1 || chatLightbox.index <= 0) return;


  const newIndex = chatLightbox.index - 1;


  const msg = chatMediaMessages[newIndex];


  setChatLightbox({ filename: msg.media!, type: msg.type as "image" | "video" | "gif", sender: msg.sender, timestamp: msg.timestamp, index: newIndex });


}


function goToChatNext() {


  if (!chatLightbox || chatMediaMessages.length <= 1 || chatLightbox.index >= chatMediaMessages.length - 1) return;


  const newIndex = chatLightbox.index + 1;


  const msg = chatMediaMessages[newIndex];


  setChatLightbox({ filename: msg.media!, type: msg.type as "image" | "video" | "gif", sender: msg.sender, timestamp: msg.timestamp, index: newIndex });


}


// Keyboard nav for chat lightbox


useEffect(() => {


  if (!chatLightbox) return;


  function handleKey(e: KeyboardEvent) {


    if (e.key === "ArrowLeft") goToChatPrev();


    else if (e.key === "ArrowRight") goToChatNext();


    else if (e.key === "Escape") setChatLightbox(null);


  }


  window.addEventListener("keydown", handleKey);


  return () => window.removeEventListener("keydown", handleKey);


}, [chatLightbox, chatMediaMessages]);


// Check for VC++ Redistributable on Windows


useEffect(() => {


  const checkVCRedist = async () => {


    try {


      const hasVC: boolean = await invoke("check_vc_redist");


      if (!hasVC) {


        setShowVCDialog(true);


      }


    } catch (err) {


      console.error("Failed to check VC++ Redistributable:", err);


    }


  };


  checkVCRedist();


}, []);


  function getInitials(name: string): string {


    return name.split(" ").map(n => n[0]).join("").toUpperCase().slice(0, 2);


  }


  const selectedChatData = chats.find(c => c.id === selectedChat);


  return (


    <>


      {/* Toast Notification - always visible */}


      {toast.visible && (


        <div style={{


          position: "fixed",


          bottom: "20px",


          right: "20px",


          background: "var(--wa-teal)",


          color: "white",


          padding: "12px 20px",


          borderRadius: "8px",


          boxShadow: "0 4px 12px rgba(0,0,0,0.3)",


          zIndex: 999999,


          fontSize: "14px",


          fontWeight: 500,


          animation: "fadeIn 0.2s ease"


        }}>


          {toast.message}


        </div>


      )}


      {showStartScreen ? (


        <StartScreen onOpenChats={() => setShowStartScreen(false)} />


      ) : (


      <div className={`app${isMobile ? ' mobile' : ''}`}>


      {linkConfirmUrl && (


        <LinkConfirmModal


          url={linkConfirmUrl}


          onConfirm={confirmOpenLink}


          onCancel={() => setLinkConfirmUrl(null)}


        />


      )}


      {showUsernameDialog && (


        <UsernameDialog


          onSubmit={handleUsernameSubmit}


          onCancel={() => setShowUsernameDialog(false)}


          hasExistingName={!!username}


        />


      )}


      {showVCDialog && (


        <div style={{


          position: 'fixed',


          top: 0,


          left: 0,


          right: 0,


          bottom: 0,


          backgroundColor: 'rgba(0, 0, 0, 0.5)',


          display: 'flex',


          alignItems: 'center',


          justifyContent: 'center',


          zIndex: 9999


        }}>


          <div style={{


            backgroundColor: theme === 'dark' ? '#1e1e1e' : '#ffffff',


            color: theme === 'dark' ? '#ffffff' : '#000000',


            padding: '32px',


            borderRadius: '12px',


            maxWidth: '500px',


            boxShadow: '0 4px 20px rgba(0, 0, 0, 0.3)'


          }}>


            <h2 style={{ marginTop: 0, marginBottom: '16px' }}>Microsoft Visual C++ Redistributable Required</h2>


            <p style={{ marginBottom: '24px', lineHeight: '1.6' }}>


              This application requires Microsoft Visual C++ Redistributable to run on Windows.


              It provides essential system libraries used by the application.


            </p>


            <p style={{ marginBottom: '24px', fontSize: '14px', opacity: 0.8 }}>


              This is a standard component from Microsoft and is safe to install.


              You will only need to install it once.


            </p>


            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>


              <button


                onClick={() => setShowVCDialog(false)}


                style={{


                  padding: '10px 20px',


                  borderRadius: '6px',


                  border: '1px solid #ccc',


                  backgroundColor: 'transparent',


                  color: theme === 'dark' ? '#ffffff' : '#000000',


                  cursor: 'pointer'


                }}


              >


                Cancel


              </button>


              <button


                onClick={async () => {


                  try {


                    await invoke('install_vc_redist');


                    setShowVCDialog(false);


                  } catch (err) {


                    console.error('Failed to install VC++ Redistributable:', err);


                    alert('Failed to start installer. Please install Visual C++ Redistributable manually from Microsoft.');


                  }


                }}


                style={{


                  padding: '10px 20px',


                  borderRadius: '6px',


                  border: 'none',


                  backgroundColor: '#007AFF',


                  color: '#ffffff',


                  cursor: 'pointer',


                  fontWeight: 'bold'


                }}


              >


                Install Now


              </button>


            </div>


          </div>


        </div>


      )}


      {deleteDialog && (


        <DeleteConfirmationDialog


          chatId={deleteDialog.chatId}


          chatName={deleteDialog.chatName}


          onSave={handleSaveChanges}


          onDelete={handleDeleteEverything}


          onCancel={handleCancelDelete}


          hasChanges={deleteDialog.hasChanges}


        />


      )}


      {successDialog && (


        <SuccessNotificationDialog


          chatName={successDialog.chatName}


          onClose={() => setSuccessDialog(null)}


        />


      )}


      {deleteSuccessDialog && (


        <DeleteSuccessDialog


          chatName={deleteSuccessDialog.chatName}


          onClose={() => setDeleteSuccessDialog(null)}


        />


      )}


      {showGroupDialog && selectedChatData?.is_group && (


        <GroupParticipantsDialog


          chatName={selectedChatData.name}


          participants={[


            ...new Set(


              messages


                .filter(m => m.type !== "system" && m.sender !== "System")


                .map(m => m.sender)


            )


          ].sort()}


          photoPath={pendingProfilePhoto ?? profile?.photo_path}


          onPhotoUpload={handleProfilePhotoUpload}


          onCancel={() => { setShowGroupDialog(false); setPendingProfilePhoto(null); }}


          onPhotoClick={(photoPath) => setChatLightbox({ filename: photoPath, type: "image", sender: "Group Photo", timestamp: "", index: 0, hideControls: true })}


          onPhotoRemove={handlePhotoRemove}


          onSave={saveProfile}


        />


      )}


      {showProfileDialog && selectedChat && (


        <ProfileDialog


          chatName={selectedChatData?.name || "Chat"}


          originalName={chats.find(c => c.id === selectedChat)?.name ?? ""}


          profileName={editingProfileName}


          profileNotes={editingProfileNotes}


          profilePhone={editingProfilePhone}


          photoPath={pendingProfilePhoto ?? profile?.photo_path}


          nameHistory={nameHistory}


          onNameChange={setEditingProfileName}


          onNotesChange={setEditingProfileNotes}


          onPhoneChange={setEditingProfilePhone}


          onSave={saveProfile}


          onCancel={() => { setShowProfileDialog(false); setPendingProfilePhoto(null); }}


          onPhotoUpload={handleProfilePhotoUpload}


          onRestoreName={handleRestoreName}


          onResetName={handleResetName}


          onPhotoClick={(photoPath) => setChatLightbox({ filename: photoPath, type: "image", sender: "Profile Photo", timestamp: "", index: 0, hideControls: true })}


          onPhotoRemove={handlePhotoRemove}


        />


      )}


      {showBackgroundModal && (


        <div className="dialog-overlay" onClick={() => { setShowBackgroundModal(false); setPendingBackground(null); }}>


          <div className="bg-modal" onClick={e => e.stopPropagation()}>


            <button className="dialog-close-btn" onClick={() => { setShowBackgroundModal(false); setPendingBackground(null); }}>×</button>


            <h3 className="bg-modal-title">Chat Background</h3>


            {/* Preview */}


            <div className="bg-modal-preview">


              {(pendingBackground === "" ? null : (pendingBackground ?? chatBackground)) ? (


                <div


                  className="bg-modal-preview-img"


                  onClick={async () => {


                    const bg = pendingBackground === "" ? null : (pendingBackground ?? chatBackground!);


                    let bgSrc = pendingBackgroundSrc && bg === pendingBackground ? pendingBackgroundSrc : null;


                    // For default backgrounds, use the path directly (served by web server)


                    // For custom backgrounds, load as base64


                    if (!bgSrc && bg && !bg.startsWith("/app_backgrounds/")) {


                      try {


                        bgSrc = await invoke<string>("read_file_as_base64", { path: bg });


                      } catch (e) {


                        console.error("Failed to load custom background for lightbox:", e);


                        // Silently fail if file doesn't exist


                      }


                    }


                    openChatLightbox({


                      media: bgSrc || bg,


                      type: "image",


                      sender: "Background",


                      timestamp: "",


                      content: "",


                      duration: null,


                      tag_ext: null,


                      caption: null,


                      display_name: null,


                      _idx: -1,


                      hideControls: true


                    });


                  }}


                  style={{ cursor: "pointer" }}


                >


                  <img


                    key={pendingBackground === "" ? null : (pendingBackground ?? chatBackground!)}


                    src={(() => {


                      const p = pendingBackground === "" ? null : (pendingBackground ?? chatBackground!);


                      if (pendingBackgroundSrc && p === pendingBackground) {


                        return pendingBackgroundSrc;


                      }


                      return p && p.startsWith("/app_backgrounds/") ? p : p ? convertFileSrc(p) : "";


                    })()}


                    alt=""


                    className="bg-modal-preview-img"


                    onError={e => { (e.target as HTMLImageElement).style.display = "none"; }}


                  />


                </div>


              ) : (


                <div className="bg-modal-preview-empty">No background selected</div>


              )}


              <span className="bg-modal-preview-label">


                {pendingBackground === "" ? "Preview (cleared)" : pendingBackground ? "Preview (unsaved)" : chatBackground ? "Current background" : ""}


              </span>


            </div>


            {/* Tabs */}


            <div className="bg-tabs">


              <button


                className={`bg-tab${bgTab === "default" ? " active" : ""}`}


                onClick={() => setBgTab("default")}


              >Default</button>


              <button


                className={`bg-tab${bgTab === "custom" ? " active" : ""}`}


                onClick={() => setBgTab("custom")}


              >Custom</button>


            </div>


            {/* Default tab */}


            {bgTab === "default" && (


              <div className="bg-tab-content">


                {defaultBackgrounds.length === 0 ? (


                  <p className="bg-empty-hint">No default backgrounds available.</p>


                ) : (


                  <div className="bg-defaults-grid">


                    {defaultBackgrounds.map(filename => (


                      <button


                        key={filename}


                        className={`bg-default-thumb${


                          (pendingBackground ?? chatBackground) === `/app_backgrounds/${filename}` ? " active" : ""


                        }`}


                        onClick={() => handleSelectDefaultBackground(filename)}


                        title={filename}


                      >


                        <img src={`/app_backgrounds/${filename}`} alt={filename} />


                      </button>


                    ))}


                  </div>


                )}


              </div>


            )}


            {/* Custom tab */}


            {bgTab === "custom" && (


              <div className="bg-tab-content">


                <button className="bg-modal-btn primary" onClick={handlePickCustomBackground}>


                  🖼️ Browse image...


                </button>


                {backgroundHistory.length > 0 && (


                  <div className="bg-history-section" style={{ marginTop: "14px" }}>


                    <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "8px" }}>


                      <button


                        type="button"


                        className="name-history-toggle"


                        onClick={() => setShowBgHistory(v => !v)}


                      >


                        {showBgHistory ? "▾" : "▸"} Previously used ({backgroundHistory.length})


                      </button>


                      <button


                        type="button"


                        className="bg-history-clear-btn"


                        onClick={handleClearBackgroundHistory}


                        title="Clear all background history"


                      >


                        🗑️ Clear


                      </button>


                    </div>


                    {showBgHistory && (


                      <div className="bg-history-list">


                        {backgroundHistory.map(entry => (


                          <div key={entry.id} className="bg-history-item">


                            <img


                              src={entry.background_path.startsWith("/") ? entry.background_path : convertFileSrc(entry.background_path)}


                              alt="Background"


                              className="bg-history-thumb"


                              onError={e => { (e.target as HTMLImageElement).style.display = "none"; }}


                            />


                            <div className="bg-history-item-info">


                              <span className="bg-history-date">{new Date(entry.changed_at).toLocaleString()}</span>


                            </div>


                            <button


                              className="bg-history-restore-btn"


                              onClick={() => handleRestoreBackground(entry.background_path)}


                              title="Select this background"


                              disabled={(pendingBackground ?? chatBackground) === entry.background_path}


                            >


                              Select


                            </button>


                            <button


                              className="bg-history-delete-btn"


                              onClick={(e) => { e.stopPropagation(); handleDeleteBackgroundEntry(entry.id); }}


                              title="Remove from history"


                            >


                              ✕


                            </button>


                          </div>


                        ))}


                      </div>


                    )}


                  </div>


                )}


              </div>


            )}


            {/* Footer actions */}


            <div className="bg-modal-footer">


              {pendingBackground !== "" && (chatBackground !== null || pendingBackground !== null) && (


                <button className="bg-modal-btn danger" onClick={handleClearBackground}>


                  Clear


                </button>


              )}


              <button


                className="bg-modal-btn primary"


                onClick={handleSaveBackground}


                disabled={pendingBackground === null || pendingBackground === chatBackground}


              >


                Save


              </button>


            </div>


          </div>


        </div>


      )}


      {exportSuccessMsg && (


        <ExportSuccessDialog


          message={exportSuccessMsg}


          onClose={() => setExportSuccessMsg(null)}


        />


      )}


      {showClearAllConfirm && (


        <div className="dialog-overlay" onClick={() => setShowClearAllConfirm(false)}>


          <div className="dialog-content" onClick={e => e.stopPropagation()}>


            <button className="dialog-close-btn" onClick={() => setShowClearAllConfirm(false)}>×</button>


            <h3>⚠️ Clear All Chats</h3>


            <p><strong>Delete all {chats.length} chat{chats.length !== 1 ? 's' : ''} permanently?</strong></p>


            <p>This cannot be undone. All messages and media will be removed.</p>


            <div className="dialog-buttons">


              <button


                className={`dialog-btn-primary ${!clearAllHasChanges ? 'disabled' : ''}`}


                onClick={handleClearAllSaveAndDelete}


                disabled={!clearAllHasChanges}


                title={!clearAllHasChanges ? 'No unsaved changes found' : 'Save edit history for all chats, then delete'}


              >


                Keep Changes & Delete All


              </button>


              <button className="dialog-btn-danger" onClick={handleClearAllChats}>Delete Everything</button>


              <button className="dialog-btn-secondary" onClick={() => setShowClearAllConfirm(false)}>Cancel</button>


            </div>


          </div>


        </div>


      )}


      {showExportMenu && (


        <div className="dialog-overlay" onClick={() => setShowExportMenu(false)}>


          <div className="dialog-content export-modal" onClick={e => e.stopPropagation()}>


            <button className="dialog-close-btn" onClick={() => setShowExportMenu(false)}>


              ×


            </button>


            <h3 style={{ textAlign: "center", marginBottom: "8px" }}>Export Chats</h3>


            <p style={{ textAlign: "center", color: "var(--text-secondary, #666)", marginBottom: "20px", fontSize: "13px" }}>


              Export your chats as a ZIP file that can be re-imported later.


            </p>


            <div className="export-modal-options">


              {selectedChat && (


                <button className="export-modal-btn" onClick={() => handleExportChat(selectedChat)}>


                  <span className="export-modal-btn-icon">💬</span>


                  <div>


                    <strong>Export Current Chat</strong>


                    <span>Save the selected chat with all messages and media</span>


                  </div>


                </button>


              )}


              <button className="export-modal-btn" onClick={handleExportAll}>


                <span className="export-modal-btn-icon">📦</span>


                <div>


                  <strong>Export All Chats</strong>


                  <span>Save all {chats.length} chats in one ZIP file</span>


                </div>


              </button>


            </div>


          </div>


        </div>


      )}


      {importing && (


        <LoadingOverlay


          message={


            importProgress && importProgress.total > 1


              ? `Importing chat ${importProgress.current} of ${importProgress.total}...`


              : "Importing chat..."


          }


          detail={


            importDetail


              ? `${importDetail.phase}${importDetail.messages > 0 ? ` · ${importDetail.messages.toLocaleString()} messages` : ''}${importDetail.media > 0 ? ` · ${importDetail.media} media files` : ''}`


              : undefined


          }


        />


      )}


      {/* Sidebar */}


      <div className="sidebar">


        <div className="sidebar-header">


          <img


            src="/icon.svg"


            alt="App Icon"


            className="app-icon-button"


            onClick={() => setShowStartScreen(true)}


            title="Return to start screen"


          />


          <h2


            className="username-header"


            onClick={() => setShowUsernameDialog(true)}


            title="Click to edit name"


          >


            {username ? (


              <>


                <span className="username-text">{username}'s</span>


                <span className="archive-text"> WhatsApp Archive</span>


              </>


            ) : (


              "WhatsApp Archive"


            )}


          </h2>


        </div>


        <div className="sidebar-toolbar">


          <button


            className="toolbar-btn"


            onClick={() => {


              setShowBackgroundModal(true);


              setPendingBackground(null);


              setBgTab("default");


              invoke<string[]>("list_default_backgrounds").then(async (files) => {
                if (files.length > 0) {
                  setDefaultBackgrounds(files);
                } else {
                  // On Android the Rust command can't read bundled assets from filesystem,
                  // fall back to the static index.json served by the WebView
                  try {
                    const res = await fetch('/app_backgrounds/index.json');
                    const list: string[] = await res.json();
                    setDefaultBackgrounds(list);
                  } catch {
                    setDefaultBackgrounds([]);
                  }
                }
              }).catch(async () => {
                try {
                  const res = await fetch('/app_backgrounds/index.json');
                  const list: string[] = await res.json();
                  setDefaultBackgrounds(list);
                } catch {
                  setDefaultBackgrounds([]);
                }
              });


            }}


            title="Change chat background"


          >


            <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">


              <rect x="3" y="3" width="18" height="18" rx="2"/>


              <circle cx="8.5" cy="8.5" r="1.5" fill="currentColor" stroke="none"/>


              <polyline points="21 15 16 10 5 21"/>


            </svg>


            Background


          </button>


          <button


            className="toolbar-btn"


            onClick={toggleTheme}


            title={theme === "light" ? "Switch to dark mode" : "Switch to light mode"}


          >


            {theme === "light" ? "🌙" : "☀️"} {theme === "light" ? "Dark" : "Light"}


          </button>


          <button className="toolbar-btn" onClick={handleImport} disabled={importing}>


            {importing ? "…" : "+ Import"}


          </button>


          <button className="toolbar-btn" onClick={() => setShowExportMenu(true)} disabled={exporting || chats.length === 0}>


            {exporting ? "…" : "Export"}


          </button>


          <button


            className="toolbar-btn toolbar-btn--danger"


            onClick={openClearAllConfirm}


            disabled={chats.length === 0}


            title="Delete all chats permanently"


          >


            🗑 Clear All


          </button>


        </div>


        {/* Global Chat Search */}


        <div className="chat-search-container">


          <input


            type="text"


            className="chat-search-input"


            placeholder="Search chats..."


            value={chatSearchQuery}


            onChange={handleChatSearch}


          />


          {chatSearchQuery && (


            <button


              className="chat-search-clear"


              onClick={() => {


                setChatSearchQuery("");


                setChatSearchResults([]);


                setIsSearchingChats(false);


                loadChatList();


              }}


            >


              ✕


            </button>


          )}


        </div>


        <div className="chat-list">


          {(isSearchingChats ? chatSearchResults : chats).length === 0 ? (


            <div className="empty-state">


              <p>No chats imported yet</p>


              <p className="hint">Click "Import" to add a WhatsApp ZIP export</p>


            </div>


          ) : (


            (isSearchingChats ? chatSearchResults : chats).map(chat => (


              <div


                key={chat.id}


                className={`chat-item ${selectedChat === chat.id ? "active" : ""}`}


                onClick={() => {


                  setSelectedChat(chat.id);


                  setSelectedMessageIndices(new Set());


                  setMultiSelectMode(false);


                  setLastSelectedIndex(null);


                  setLastLongPressedIndex(null);


                }}


              >


                {chat.photo_path ? (


                  <ProfileImage photoPath={chat.photo_path} alt={chat.name} className="chat-avatar chat-avatar--photo" />


                ) : chat.is_group ? (


                  <GroupAvatar


                    participants={chat.name.replace(/ \(Group\)$/, "").split(/[,&]+/).map(s => s.trim()).filter(Boolean)}


                    size="normal"


                  />


                ) : (


                  <div className="chat-avatar">


                    {getInitials(chat.name)}


                  </div>


                )}


                <div className="chat-info">


                  <div className="chat-row">


                    <span className="chat-name">{chat.name}</span>


                    {chat.is_group && <span className="group-badge">Group</span>}


                    <div className="chat-timestamp-col">


                      <span className="chat-date">{formatDate(chat.timestamp)}</span>


                      <span className="chat-time">{formatTime(chat.timestamp)}</span>


                    </div>


                  </div>


                  <div className="chat-row">


                    <span className="chat-preview">{chat.last_message.slice(0, 40)}</span>


                    <button 


                      className="delete-btn"


                      onClick={(e) => handleDelete(chat.id, e)}


                    >


                      Delete


                    </button>


                  </div>


                </div>


              </div>


            ))


          )}


        </div>


      </div>


      {/* Chat View */}


      <div className={`chat-view${isMobile && selectedChat ? ' active' : ''}`}>


        {selectedChat ? (


          <>


            <div className="chat-header">


              <div className="chat-header-left">

                {isMobile && (
                  <button
                    className="mobile-back-btn"
                    onClick={() => setSelectedChat(null)}
                    title="Back"
                  >
                    ‹
                  </button>
                )}


                {selectedChatData?.is_group ? (


                  profile?.photo_path ? (


                    <ProfileImage photoPath={profile.photo_path} alt="Group" className="chat-avatar large chat-avatar--photo" />


                  ) : (


                    <GroupAvatar


                      participants={[


                        ...new Set(


                          messages


                            .filter(m => m.type !== "system" && m.sender !== "System")


                            .map(m => m.sender)


                        )


                      ]}


                      size="large"


                    />


                  )


                ) : (


                  profile?.photo_path ? (


                    <ProfileImage photoPath={profile.photo_path} alt="Profile" className="chat-avatar large chat-avatar--photo" />


                  ) : (


                    <div className="chat-avatar large">


                      {getInitials(selectedChatData?.name || "Chat")}


                    </div>


                  )


                )}


                <div className="chat-header-info">


                  <h3>{selectedChatData?.name}</h3>


                  <span>{selectedChatData?.is_group ? "Group" : "Personal"}</span>


                </div>


              </div>


              {/* Multi-select action buttons */}


              {selectedMessageIndices.size > 0 && (


                <>


                  <button


                    className="media-gallery-btn"


                    style={{ padding: '4px 8px', borderRadius: '6px', fontSize: '13px', whiteSpace: 'nowrap' }}


                    onClick={() => {


                      const selectedIndices = Array.from(selectedMessageIndices);


                      Promise.all(selectedIndices.map(idx => {


                        const newFavorite = !messages[idx].is_favorite;


                        return invoke("toggle_message_favorite", {


                          chatId: selectedChat,


                          messageIdx: idx,


                          isFavorite: newFavorite


                        });


                      })).then(() => {


                        const updatedMessages = [...messages];


                        selectedIndices.forEach(idx => {


                          updatedMessages[idx] = { ...updatedMessages[idx], is_favorite: !messages[idx].is_favorite };


                        });


                        setMessages(updatedMessages);


                        setSelectedMessageIndices(new Set());


                        setMultiSelectMode(false);


                        setLastSelectedIndex(null);


                        setLastLongPressedIndex(null);


                      }).catch(console.error);


                    }}


                    title="Toggle favorites"


                  >


                    {(() => {
                      const indices = Array.from(selectedMessageIndices);
                      const allFav = indices.every(i => messages[i]?.is_favorite);
                      const noneFav = indices.every(i => !messages[i]?.is_favorite);
                      const n = selectedMessageIndices.size;
                      if (allFav) return `⭐− ${n}`;
                      if (noneFav) return `⭐+ ${n}`;
                      return `⭐~ ${n}`;
                    })()}


                  </button>


                  <button


                    className="media-gallery-btn"


                    style={{ padding: '4px 8px', borderRadius: '6px', fontSize: '13px' }}


                    onClick={() => {


                      setSelectedMessageIndices(new Set());


                      setMultiSelectMode(false);


                      setLastSelectedIndex(null);


                      setLastLongPressedIndex(null);


                    }}


                    title="Deselect all"


                  >


                    ✕


                  </button>


                </>


              )}


              <button


                className="search-btn"


                onClick={() => { setShowMessageSearch(v => !v); setShowMediaGallery(false); setShowFavorites(false); setShowProfileDialog(false); setShowGroupDialog(false); }}


                title="Search messages"


              >


                🔍


              </button>


              <button


                className="media-gallery-btn"


                onClick={() => { setShowMediaGallery(v => !v); setShowMessageSearch(false); setShowFavorites(false); setShowProfileDialog(false); setShowGroupDialog(false); }}


                title="Media, links &amp; docs"


              >


                <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">


                  <rect x="3" y="3" width="18" height="18" rx="2"/>


                  <circle cx="8.5" cy="8.5" r="1.5"/>


                  <polyline points="21 15 16 10 5 21"/>


                </svg>


              </button>


              <button


                className="favorites-btn"


                onClick={() => {


                  setShowFavorites(v => !v);


                  setShowMessageSearch(false);


                  setShowMediaGallery(false);


                  setShowProfileDialog(false);


                  setShowGroupDialog(false);


                  if (!showFavorites && selectedChat) {


                    invoke<{ messages: Message[] }>("get_favorite_messages", { chatId: selectedChat })


                      .then(result => {


                        const messagesWithIndex = result.messages.map(fav => ({
                          ...fav,
                          _idx: fav.id ?? -1
                        }));

                        console.log("Favorites debug:", {
                          totalMessages: messages.length,
                          favoritesCount: result.messages.length,
                          firstFav: result.messages[0],
                          firstFavIdx: result.messages[0]?.id
                        });


                        setFavoriteMessages(messagesWithIndex);


                      })


                      .catch(console.error);


                  }


                }}


                title="Starred messages"


              >


                <svg viewBox="0 0 24 24" width="22" height="22" fill={showFavorites ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">


                  <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>


                </svg>


              </button>


              <button


                className="profile-btn"


                onClick={() => { setShowFavorites(false); setShowMediaGallery(false); setShowMessageSearch(false); if (selectedChatData?.is_group) { setShowGroupDialog(true); } else { setShowProfileDialog(true); if (selectedChat) loadNameHistory(selectedChat); } }}


                title={selectedChatData?.is_group ? "View participants" : "Edit profile"}


              >


                <svg viewBox="0 0 24 24" className="profile-btn-icon">


                  <circle cx="12" cy="8" r="4" fill="#e6b800"/>


                  <path d="M4,20 Q4,14 12,14 Q20,14 20,20" fill="#e6b800" stroke="none"/>


                </svg>


              </button>


            </div>


            {showMessageSearch && (


              <div className="search-overlay" ref={searchOverlayRef}>


                <div className="search-box">


                  <input


                    type="text"


                    placeholder="Search messages..."


                    value={showAdvancedSearch ? searchFilters.query : messageSearchQuery}


                    onChange={(e) => {


                      if (showAdvancedSearch) {


                        setSearchFilters({...searchFilters, query: e.target.value});


                      } else {


                        handleMessageSearchInput(e);


                      }


                    }}


                    onKeyDown={(e) => {


                      if (e.key === "Enter") {


                        if (showAdvancedSearch) {


                          handleAdvancedSearch();


                        } else {


                          handleMessageSearch();


                        }


                      } else if (e.key === "Escape") {


                        setShowMessageSearch(false);


                      }


                    }}


                    autoFocus


                  />


                  <button 


                    className={`search-advanced-btn ${showAdvancedSearch ? 'active' : ''}`}


                    title="Advanced search"


                    onClick={() => setShowAdvancedSearch(!showAdvancedSearch)}


                  ><svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><line x1="4" y1="6" x2="20" y2="6"/><line x1="6" y1="12" x2="18" y2="12"/><line x1="9" y1="18" x2="15" y2="18"/></svg></button>


                  {messageSearchResults.length > 0 && (


                    <div className="search-nav">


                      <span className="search-nav-count">


                        {searchResultCursor + 1}/{messageSearchResults.length}


                      </span>


                      <button


                        className="search-nav-btn"


                        title="Previous result"


                        onClick={() => jumpToResult(searchResultCursor - 1)}


                      >↑</button>


                      <button


                        className="search-nav-btn"


                        title="Next result"


                        onClick={() => jumpToResult(searchResultCursor + 1)}


                      >↓</button>


                    </div>


                  )}


                  <button className="search-close-btn" onClick={() => {


                    setShowMessageSearch(false);


                    setHighlightedIndices(new Set());


                    setJumpedIndex(null);


                    setMessageSearchResults([]);


                    setMessageSearchQuery("");


                    setSearchFilters({


                      query: "",


                      date_from: null,


                      date_to: null,


                      sender: null,


                      msg_type: null,


                    });


                    setSearchResultCursor(0);


                    setShowAdvancedSearch(false);


                  }}>✕</button>


                </div>


                {showAdvancedSearch && (


                  <div className="search-filters">


                    <div className="filter-row">


                      <div className="filter-group">


                        <label>From Date:</label>


                        <input


                          type="date"


                          value={searchFilters.date_from || ""}


                          onChange={(e) => setSearchFilters({...searchFilters, date_from: e.target.value || null})}


                        />


                      </div>


                      <div className="filter-group">


                        <label>To Date:</label>


                        <input


                          type="date"


                          value={searchFilters.date_to || ""}


                          onChange={(e) => setSearchFilters({...searchFilters, date_to: e.target.value || null})}


                        />


                      </div>


                    </div>


                    <div className="filter-row">


                      <div className="filter-group">


                        <label>Sender:</label>


                        <input


                          type="text"


                          placeholder="Filter by sender..."


                          value={searchFilters.sender || ""}


                          onChange={(e) => setSearchFilters({...searchFilters, sender: e.target.value || null})}


                        />


                      </div>


                      <div className="filter-group">


                        <label>Message Type:</label>


                        <select


                          value={searchFilters.msg_type || ""}


                          onChange={(e) => setSearchFilters({...searchFilters, msg_type: e.target.value || null})}


                        >


                          <option value="">All Types</option>


                          <option value="text">Text</option>


                          <option value="image">Image</option>


                          <option value="video">Video</option>


                          <option value="audio">Audio</option>


                          <option value="file">File</option>


                          <option value="system">System</option>


                        </select>


                      </div>


                    </div>


                    <div className="filter-actions">


                      <button className="filter-search-btn" onClick={handleAdvancedSearch}>


                        Search with Filters


                      </button>


                      <button 


                        className="filter-clear-btn" 


                        onClick={() => setSearchFilters({


                          query: searchFilters.query,


                          date_from: null,


                          date_to: null,


                          sender: null,


                          msg_type: null,


                        })}


                      >


                        Clear Filters


                      </button>


                    </div>


                  </div>


                )}


                {messageSearchResults.length > 0 && (


                  <div className="search-results">


                    <div className="search-count">{messageSearchResults.length} result{messageSearchResults.length !== 1 ? 's' : ''} found</div>


                    {messageSearchResults.map((result, pos) => (


                      <div


                        key={result.message_index}


                        className={`search-result-item${pos === searchResultCursor ? ' active' : ''}`}


                        onClick={() => jumpToMessage(result.message_index)}


                      >


                        <div className="search-result-header">


                          <span className="search-result-sender">{result.sender}</span>


                          <span className="search-result-time">{formatDate(result.timestamp)} {formatTime(result.timestamp)}</span>


                        </div>


                        <div className="search-result-content">


                          {highlightText(


                            result.content.slice(0, 100) + (result.content.length > 100 ? "..." : ""),


                            messageSearchQuery


                          )}


                        </div>


                      </div>


                    ))}


                  </div>


                )}


                {messageSearchQuery && messageSearchResults.length === 0 && (


                  <div className="search-no-results">No messages found</div>


                )}


              </div>


            )}


            {showMediaGallery && (


              <MediaGallery


                messages={messages}


                chatId={selectedChat}


                activeTab={mediaGalleryTab}


                onTabChange={setMediaGalleryTab}


                onClose={() => setShowMediaGallery(false)}


                onJumpToMessage={(index) => {


                  setShowMediaGallery(false);


                  setShowFavorites(false);


                  setJumpedIndex(index);


                  scrollToResult(index);


                }}


              />


            )}


            <ErrorBoundary>


            <>


            <div className={`messages-container${chatBackground ? " messages-container--has-bg" : ""}`} style={showMediaGallery || showFavorites ? { display: "none" } : chatBackground ? { backgroundImage: `url('${chatBackground.startsWith("/app_backgrounds/") ? chatBackground : convertFileSrc(chatBackground)}')`, backgroundSize: "cover", backgroundPosition: "center" } : {}} ref={messagesContainerRef}>


              {loading ? (


                <div className="loading">Loading messages...</div>


              ) : messages.length === 0 ? (


                <div className="empty-messages">


                  <div className="empty-messages-content">


                    <p>No messages in this chat yet</p>


                  </div>


                </div>


              ) : (


                <>


                  <VirtualMessageList


                    ref={virtualListRef}


                    messages={messages}


                    followOutput={followOutput}


                    renderMessage={(_msg, idx) => {


                    const msg = _msg as Message;


                    const isMe = msg.sender === username;


                    const isSystem = msg.type === "system";


                    const showDate = idx === 0 || 


                      formatDate(msg.timestamp) !== formatDate(messages[idx - 1].timestamp);


                    // Media type detection - recomputes on every render with fresh msg values


                    const isFileMsg = msg.type === "file" && msg.media;


                    const tagExtTrimmed = (msg.tag_ext ?? "").trim();


                    const effExt = isFileMsg ? (tagExtTrimmed || msg.media?.split(".").pop() || "").toLowerCase().replace(/^\./, "") : "";


                    const isImageFile = isFileMsg && IMAGE_EXTS.has(effExt);


                    const isVideoFile = isFileMsg && VIDEO_EXTS.has(effExt);


                    const isAudioFile = isFileMsg && AUDIO_EXTS.has(effExt);


                    if (isFileMsg && msg.media?.includes("DOC-20260504")) {


                    }


                    return (


                      <div key={`${idx}-${msg.media}`}>


                        {showDate && (


                          <div className="date-divider" data-date={formatDate(msg.timestamp)} />


                        )}


                        {isSystem ? (


                          <div className="system-message">{msg.content}</div>


                        ) : (


                          <div


                            ref={(el) => { messageRefs.current[idx] = el; }}


                            className={`message ${isMe ? "sent" : "received"}${msg.type === "sticker" ? " sticker-message" : ""}${highlightedIndices.has(idx) ? " search-match" : ""}${jumpedIndex === idx ? " jumped" : ""}${selectedMessageIndices.has(idx) ? " selected" : ""}`}


                            onMouseDown={(_e) => {


                              // Start long-press timer


                              const timer = setTimeout(() => {


                                setMultiSelectMode(true);


                                const newSelected = new Set(selectedMessageIndices);


                                newSelected.add(idx);


                                setSelectedMessageIndices(newSelected);


                                setLastSelectedIndex(idx);


                                setLastLongPressedIndex(idx); // Track this message


                                // Clear timer after it fires


                                setPressTimers(new Map(pressTimers).set(idx, undefined as any));


                              }, 500); // 500ms hold (standard long-press)


                              setPressTimers(new Map(pressTimers).set(idx, timer));


                            }}


                            onMouseUp={() => {


                              // Clear timer on mouse up (only if hasn't fired yet)


                              const timer = pressTimers.get(idx);


                              if (timer) {


                                clearTimeout(timer);


                                setPressTimers(new Map(pressTimers).set(idx, undefined as any));


                              }


                            }}


                            onMouseLeave={() => {


                              // Clear timer on mouse leave (only if hasn't fired yet)


                              const timer = pressTimers.get(idx);


                              if (timer) {


                                clearTimeout(timer);


                                setPressTimers(new Map(pressTimers).set(idx, undefined as any));


                              }


                            }}

                            onTouchStart={(e) => {
                              touchScrollingRef.current = false;
                              const touch = e.touches[0];
                              touchStartPos.current = { x: touch.clientX, y: touch.clientY };
                              const timer = setTimeout(() => {
                                setMultiSelectMode(true);
                                const newSelected = new Set(selectedMessageIndices);
                                if (newSelected.has(idx)) {
                                  newSelected.delete(idx);
                                  if (newSelected.size === 0) {
                                    setMultiSelectMode(false);
                                    setLastSelectedIndex(null);
                                  }
                                } else {
                                  newSelected.add(idx);
                                  setLastSelectedIndex(idx);
                                }
                                setSelectedMessageIndices(newSelected);
                                setLastLongPressedIndex(idx);
                                setPressTimers(new Map(pressTimers).set(idx, undefined as any));
                              }, 500);
                              setPressTimers(new Map(pressTimers).set(idx, timer));
                            }}

                            onTouchEnd={() => {
                              const timer = pressTimers.get(idx);
                              if (timer) { clearTimeout(timer); setPressTimers(new Map(pressTimers).set(idx, undefined as any)); }
                              if (lastLongPressedIndex === idx) {
                                setLastLongPressedIndex(null);
                              }
                            }}

                            onTouchMove={(e) => {
                              const touch = e.touches[0];
                              if (touchStartPos.current) {
                                const dx = Math.abs(touch.clientX - touchStartPos.current.x);
                                const dy = Math.abs(touch.clientY - touchStartPos.current.y);
                                if (dx > 10 || dy > 10) {
                                  touchScrollingRef.current = true;
                                }
                              }
                              const timer = pressTimers.get(idx);
                              if (timer) { clearTimeout(timer); setPressTimers(new Map(pressTimers).set(idx, undefined as any)); }
                            }}

                            onClick={(e) => {


                              if (touchScrollingRef.current) return;


                              if (!e.defaultPrevented) {


                                if (multiSelectMode) {


                                  // In multi-select mode, clicks do nothing - only long-press works


                                  e.preventDefault();


                                  return;


                                }


                                // Normal click does nothing in non-multi-select mode


                              }


                            }}


                          >


                            {selectedChatData?.is_group && (


                              <span className="sender-name">{msg.sender}{isMe ? " (You)" : ""}</span>


                            )}


                            {msg.type === "image" && msg.media && (


                              <div className="media-content image" onClick={(e) => {


                                // Skip if this was just long-pressed


                                if (lastLongPressedIndex === idx) {


                                  setLastLongPressedIndex(null);


                                  e.preventDefault();


                                  e.stopPropagation();


                                  return;


                                }


                                if (multiSelectMode) {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  const newSelected = new Set(selectedMessageIndices);


                                  if (newSelected.has(idx)) {


                                    newSelected.delete(idx);


                                    if (newSelected.size === 0) {


                                      setLastSelectedIndex(null);


                                      setMultiSelectMode(false);


                                    }


                                  } else {


                                    newSelected.add(idx);


                                    setLastSelectedIndex(idx);


                                  }


                                  setSelectedMessageIndices(newSelected);


                                } else {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  openChatLightbox({ ...msg, _idx: idx });


                                }


                              }} style={{ cursor: "pointer" }}>


                                <LazyMediaImage


                                  chatId={selectedChat}


                                  index={idx}


                                  filename={msg.media}


                                  onLoad={(e) => {


                                    (e.target as HTMLImageElement).parentElement?.classList.add("loaded");


                                  }}


                                />


                              </div>


                            )}


                            {msg.type === "sticker" && msg.media && (


                              <div className="media-content sticker">


                                <StickerImage


                                  chatId={selectedChat}


                                  filename={msg.media}


                                />


                              </div>


                            )}


                            {msg.type === "video" && msg.media && selectedChat && (


                              <div className="media-content video" onClick={(e) => {


                                // Skip if this was just long-pressed


                                if (lastLongPressedIndex === idx) {


                                  setLastLongPressedIndex(null);


                                  e.preventDefault();


                                  e.stopPropagation();


                                  return;


                                }


                                if (multiSelectMode) {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  const newSelected = new Set(selectedMessageIndices);


                                  if (newSelected.has(idx)) {


                                    newSelected.delete(idx);


                                    if (newSelected.size === 0) {


                                      setLastSelectedIndex(null);


                                      setMultiSelectMode(false);


                                    }


                                  } else {


                                    newSelected.add(idx);


                                    setLastSelectedIndex(idx);


                                  }


                                  setSelectedMessageIndices(newSelected);


                                } else {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  openChatLightbox({ ...msg, _idx: idx });


                                }


                              }} style={{ cursor: "pointer" }}>


                                <VideoPlayer


                                  chatId={selectedChat}


                                  index={idx}


                                  filename={msg.media}


                                  onToggleType={() => {


                                    invoke("set_message_type", { chatId: selectedChat, messageIdx: idx, msgType: "gif" }).catch(console.error);


                                    setMessages(prev => prev.map((m, i) => i === idx ? { ...m, type: "gif" } : m));


                                    showToast("Changed to GIF");


                                  }}


                                />


                              </div>


                            )}


                            {msg.type === "gif" && msg.media && selectedChat && (


                              <div className="media-content gif" onClick={(e) => {


                                // Skip if this was just long-pressed


                                if (lastLongPressedIndex === idx) {


                                  setLastLongPressedIndex(null);


                                  e.preventDefault();


                                  e.stopPropagation();


                                  return;


                                }


                                if (multiSelectMode) {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  const newSelected = new Set(selectedMessageIndices);


                                  if (newSelected.has(idx)) {


                                    newSelected.delete(idx);


                                    if (newSelected.size === 0) {


                                      setLastSelectedIndex(null);


                                      setMultiSelectMode(false);


                                    }


                                  } else {


                                    newSelected.add(idx);


                                    setLastSelectedIndex(idx);


                                  }


                                  setSelectedMessageIndices(newSelected);


                                } else {


                                  e.preventDefault();


                                  e.stopPropagation();


                                  openChatLightbox({ ...msg, _idx: idx });


                                }


                              }} style={{ cursor: "pointer" }}>


                                <GifPlayer


                                  chatId={selectedChat}


                                  index={idx}


                                  filename={msg.media}


                                  onToggleType={() => {


                                    invoke("set_message_type", { chatId: selectedChat, messageIdx: idx, msgType: "video" }).catch(console.error);


                                    setMessages(prev => prev.map((m, i) => i === idx ? { ...m, type: "video" } : m));


                                    showToast("Changed to Video");


                                  }}


                                />


                              </div>


                            )}


                            {msg.type === "audio" && msg.media && (


                              <div className="media-content audio">


                                <AudioBase64 chatId={selectedChat} filename={msg.media} />


                                {msg.duration && <span className="audio-duration">{msg.duration}</span>}


                              </div>


                            )}


                            {msg.type === "location" && msg.media && (


                              <div className="media-content location-attachment">


                                <span className="location-icon">📍</span>


                                <div className="location-info">


                                  <span className="location-label">Location</span>


                                  <span className="location-url">{msg.media}</span>


                                </div>


                                <button


                                  className="file-open-btn"


                                  title="Open in maps"


                                  onClick={() => openLinkWithConfirm(msg.media!)}


                                >


                                  Open Maps


                                </button>


                              </div>


                            )}


                            {/* Media display using precomputed detection variables */}


                            {isImageFile && (


                              <FileImageInline key={`img-${idx}-${msg.media}-${effExt}`} chatId={selectedChat} filename={msg.media!} tagExt={msg.tag_ext ?? null} onClick={() => openChatLightbox({ ...msg, _idx: idx })} />


                            )}


                            {isVideoFile && selectedChat && (


                              <div key={`vid-${idx}-${msg.media}-${effExt}`} className="media-content video" onClick={() => openChatLightbox({ ...msg, _idx: idx })} style={{ cursor: "pointer" }}>


                                <VideoPlayer chatId={selectedChat} index={idx} filename={msg.media!} onToggleType={undefined} />


                              </div>


                            )}


                            {isAudioFile && (


                              <div key={`aud-${idx}-${msg.media}-${effExt}`} className="media-content audio">


                                <AudioBase64 chatId={selectedChat} filename={msg.media!} />


                              </div>


                            )}


                            {isFileMsg && (


                              <FileAttachment


                                msg={msg}


                                idx={idx}


                                chatId={selectedChat}


                                onTagSaved={(patch) => setMessages(prev => prev.map((m, i) => i === idx ? { ...m, ...patch } : m))}


                                showToast={showToast}


                              />


                            )}


                            {msg.type === "poll" && (


                              <PollMessage content={msg.content} />


                            )}


                            {msg.type === "event" && (


                              <EventMessage content={msg.content} />


                            )}


                            {/* Fallback for messages with media that failed to load or don't match known types */}


                            {msg.media && !["image","video","gif","audio","location","file","sticker"].includes(msg.type) && (


                              <MediaFallback filename={msg.media} />


                            )}


                            {/* Fallback for completely empty messages */}


                            {!msg.media && !msg.content && (


                              <MediaFallback filename="(missing file)" />


                            )}


                            {msg.content && msg.content !== `<Media omitted>` && 


                             msg.type !== "sticker" &&


                             msg.type !== "poll" &&


                             msg.type !== "event" &&


                             !msg.content.includes("(file attached)") &&


                             !msg.content.includes("(bestand bijgevoegd)") &&


                             !msg.content.includes("<Media weggelaten>") && (


                              <ExpandableText


                                content={msg.content}


                                highlight={highlightedIndices.has(idx) && messageSearchQuery ? messageSearchQuery : undefined}


                                renderFn={renderMessageText} 


                              />


                            )}


                            <span className="message-time">


                              {formatTime(msg.timestamp)}


                              {msg.is_favorite && <span className="message-star-indicator">⭐</span>}


                            </span>


                          </div>


                        )}


                      </div>


                    );


                  }}


                  />


                  <div ref={messagesEndRef} />


                </>


              )}


              </div>


              {showFavorites && (


                <FavoritesGallery


                  messages={favoriteMessages}


                  chatId={selectedChat || ""}


                  onClose={() => setShowFavorites(false)}


                  onJumpToMessage={jumpToMessage}


                  username={username}


                  selectedChatData={selectedChatData}


                  selectedMessageIndices={selectedMessageIndices}


                  setSelectedMessageIndices={setSelectedMessageIndices}


                  highlightedIndices={highlightedIndices}


                  jumpedIndex={jumpedIndex}


                  openChatLightbox={openChatLightbox}


                />


              )}


              {/* Jump buttons for chat */}


              <div className="jump-buttons-container">


                <JumpButton 


                  direction="up" 


                  onClick={scrollToChatTop}


                  visible={messages.length > 10 && !showFavorites && !showMediaGallery && !showMessageSearch}


                />


                <JumpButton 


                  direction="down" 


                  onClick={scrollToChatBottom}


                  visible={messages.length > 10 && !showFavorites && !showMediaGallery && !showMessageSearch}


                />


              </div>


            </>


            </ErrorBoundary>


          </>


        ) : (


          <div className="empty-chat">


            <div className="empty-content">


              <h2>WhatsApp Archive Viewer</h2>


              <p>Select a chat from the sidebar or import a new one</p>


              <button className="import-btn large" onClick={handleImport} disabled={importing}>


                {importing ? "Importing..." : "Import WhatsApp ZIP"}


              </button>


            </div>


          </div>


        )}


      </div>


      {/* Chat Media Lightbox */}


      {chatLightbox && (


        <div className="lightbox-overlay" onClick={() => setChatLightbox(null)}>


          <button className="lightbox-close" onClick={() => setChatLightbox(null)}>✕</button>


          {chatMediaMessages.length > 1 && (


            <>


              {!chatLightbox.hideControls && <button className={`lightbox-nav lightbox-nav--prev${chatLightbox.index <= 0 ? " disabled" : ""}`} onClick={(e) => { e.stopPropagation(); goToChatPrev(); }} disabled={chatLightbox.index <= 0}>‹</button>}


              {!chatLightbox.hideControls && <button className={`lightbox-nav lightbox-nav--next${chatLightbox.index >= chatMediaMessages.length - 1 ? " disabled" : ""}`} onClick={(e) => { e.stopPropagation(); goToChatNext(); }} disabled={chatLightbox.index >= chatMediaMessages.length - 1}>›</button>}


            </>


          )}


          <div className="lightbox-meta" onClick={(e) => e.stopPropagation()}>


            <span className="lightbox-sender">{chatLightbox.sender}</span>


            <span className="lightbox-timestamp">{chatLightbox.timestamp}</span>


            {!chatLightbox.hideControls && (


            <button


              className="lightbox-fav-btn"


              onClick={(e) => {


                e.stopPropagation();


                const currentMsg = chatMediaMessages[chatLightbox.index];


                if (currentMsg) {


                  const newFavorite = !currentMsg.is_favorite;


                  console.log("Toggling favorite:", { selectedChat, messageIdx: currentMsg._idx, newFavorite });


                  invoke("toggle_message_favorite", {


                    chatId: selectedChat,


                    messageIdx: currentMsg._idx,


                    isFavorite: newFavorite


                  }).then(() => {


                    const updatedMessages = [...messages];


                    updatedMessages[currentMsg._idx] = { ...currentMsg, is_favorite: newFavorite };


                    setMessages(updatedMessages);


                    console.log("Favorite toggled successfully");


                  }).catch(err => {


                    console.error("Failed to toggle favorite:", err);


                  });


                }


              }}


              title={chatMediaMessages[chatLightbox.index]?.is_favorite ? "Remove from favorites" : "Add to favorites"}


            >


              <svg viewBox="0 0 24 24" width="24" height="24" fill={chatMediaMessages[chatLightbox.index]?.is_favorite ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">


                <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/>


              </svg>


            </button>


            )}


          </div>


          {chatLightbox.type === "image" ? (


            chatLightbox.hideControls ? (


              <img


                src={chatLightbox.filename.startsWith("data:") || chatLightbox.filename.startsWith("/app_backgrounds/") ? chatLightbox.filename : convertFileSrc(chatLightbox.filename)}


                alt=""


                className="lightbox-img"


              />


            ) : (


              <MediaImage


                chatId={selectedChat!}


                filename={chatLightbox.filename}


                alt=""


                className="lightbox-img"


              />


            )


          ) : chatLightbox.type === "gif" ? (


            <div onClick={(e) => e.stopPropagation()}>


              <GifPlayer chatId={selectedChat!} filename={chatLightbox.filename} />


            </div>


          ) : (


            <div onClick={(e) => e.stopPropagation()}>


              <VideoPlayer chatId={selectedChat!} filename={chatLightbox.filename} controls className="lightbox-video" />


            </div>


          )}


        </div>


      )}


    </div>


    )}


    </>


  );


}


export default App;

