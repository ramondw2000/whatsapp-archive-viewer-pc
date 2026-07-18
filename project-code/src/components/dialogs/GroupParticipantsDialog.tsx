import { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { GroupAvatar } from "../GroupAvatar";

interface GroupParticipantsDialogProps {
  chatName: string;
  participants: string[];
  photoPath: string | null | undefined;
  onPhotoUpload: () => void;
  onCancel: () => void;
  onPhotoClick: (base64Data: string) => void;
  onPhotoRemove: () => void;
  onSave: () => void;
  linkedParticipants?: Set<string>;
  onEditParticipant?: (name: string) => void;
  formerMembers?: Set<string>;
  displayNameOverrides?: Record<string, string>;
  participantPhotos?: Record<string, string>;
  photoHistoryCount?: number;
  onOpenPhotoHistory?: () => void;
}

export function GroupParticipantsDialog({
  chatName,
  participants,
  photoPath,
  onPhotoUpload,
  onCancel,
  onPhotoClick,
  onPhotoRemove,
  onSave,
  linkedParticipants,
  onEditParticipant,
  formerMembers,
  displayNameOverrides = {},
  participantPhotos = {},
  photoHistoryCount,
  onOpenPhotoHistory,
}: GroupParticipantsDialogProps) {
  const [photoSrc, setPhotoSrc] = useState<string | null>(null);

  useEffect(() => {
    if (!photoPath) {
      setPhotoSrc(null);
      return;
    }
    invoke<string>("read_file_as_base64", { path: photoPath })
      .then(setPhotoSrc)
      .catch(() => setPhotoSrc(null));
  }, [photoPath]);

  // Participant avatars are loaded as base64 too, same as the group's own photo above —
  // convertFileSrc doesn't reliably resolve paths under profile_photos in this app.
  const [participantPhotoSrcs, setParticipantPhotoSrcs] = useState<Record<string, string>>({});

  useEffect(() => {
    let cancelled = false;
    Object.entries(participantPhotos).forEach(([name, path]) => {
      if (participantPhotoSrcs[name]) return;
      invoke<string>("read_file_as_base64", { path })
        .then(src => { if (!cancelled) setParticipantPhotoSrcs(prev => ({ ...prev, [name]: src })); })
        .catch(() => {});
    });
    return () => { cancelled = true; };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [participantPhotos]);

  function getInitials(name: string): string {
    // Array.from splits by Unicode code point rather than UTF-16 code unit, so surrogate-pair
    // emoji (e.g. most emoji outside the Basic Multilingual Plane) aren't cut in half.
    return name
      .split(" ")
      .filter(Boolean)
      .slice(0, 2)
      .map(word => Array.from(word)[0] ?? "")
      .join("")
      .toUpperCase();
  }

  return (
    <div className="dialog-overlay">
      <div className="dialog-content profile-dialog">
        <h3>Group Info</h3>
        <p className="profile-chat-name">{chatName}</p>
        <div className="profile-photo-section">
          {photoSrc ? (
            <img src={photoSrc} alt="Group" className="profile-photo" onClick={() => onPhotoClick(photoSrc)} style={{ cursor: 'pointer' }} />
          ) : (
            participants.length > 0 ? (
              <GroupAvatar participants={participants} size="dialog" />
            ) : (
              <div className="profile-photo-placeholder">
                <svg viewBox="0 0 100 100" className="profile-icon">
                  <circle cx="35" cy="38" r="16" fill="#e6b800"/>
                  <circle cx="65" cy="38" r="16" fill="#e6b800"/>
                  <path d="M5,85 Q5,60 35,60 Q50,60 50,60 Q50,60 65,60 Q95,60 95,85 L95,95 L5,95 Z" fill="#e6b800"/>
                </svg>
              </div>
            )
          )}
          <button type="button" className="upload-photo-btn" onClick={onPhotoUpload}>
            📷 Upload Group Photo
          </button>
          {photoPath && (
            <button type="button" className="upload-photo-btn remove-photo-btn" onClick={onPhotoRemove}>
              🗑️ Remove Photo
            </button>
          )}
          {!!photoHistoryCount && onOpenPhotoHistory && (
            <button type="button" className="upload-photo-btn" onClick={onOpenPhotoHistory}>
              🖼️ Photo History ({photoHistoryCount})
            </button>
          )}
        </div>
        <p className="group-participant-count">{participants.length} participant{participants.length !== 1 ? "s" : ""}</p>
        <div className="group-participants-list">
          {participants.map(name => {
            const displayName = displayNameOverrides[name] ?? name;
            const linkedPhotoSrc = participantPhotoSrcs[name];
            return (
            <div key={name} className="group-participant-item">
              {linkedPhotoSrc ? (
                <img
                  src={linkedPhotoSrc}
                  alt={displayName}
                  className="group-participant-avatar group-participant-avatar-img"
                />
              ) : (
                <div className="group-participant-avatar">{getInitials(displayName)}</div>
              )}
              <span className="group-participant-name">{displayName}</span>
              {formerMembers?.has(name) && (
                <span className="former-member-badge" title="No longer in this group">Former member</span>
              )}
              {linkedParticipants?.has(name) && (
                <span className="group-participant-linked-icon" title="Linked to a 1-on-1 chat">🔗</span>
              )}
              {onEditParticipant && (
                <button
                  type="button"
                  className="group-participant-edit-btn"
                  title="Edit profile"
                  onClick={() => onEditParticipant(name)}
                >
                  ✏️
                </button>
              )}
            </div>
            );
          })}
        </div>
        <div className="dialog-buttons">
          <button type="button" onClick={onCancel}>
            Cancel
          </button>
          <button type="button" onClick={onSave} className="primary">
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
