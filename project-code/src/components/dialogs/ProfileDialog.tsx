import { useState, useEffect, useRef } from 'react';
import { invoke } from "@tauri-apps/api/core";
import { parsePhoneNumber, formatPhoneNumber, validatePhoneInput, getPlaceholder, phoneOptions } from "../../utils/phoneNumber";
import type { NameHistoryEntry } from "../../types";

interface ProfileDialogProps {
  chatName: string;
  originalName: string;
  profileName: string;
  profileNotes: string;
  profilePhone: string;
  photoPath: string | null | undefined;
  nameHistory: NameHistoryEntry[];
  onNameChange: (name: string) => void;
  onNotesChange: (notes: string) => void;
  onPhoneChange: (phone: string) => void;
  onSave: () => void;
  onCancel: () => void;
  onPhotoUpload: () => void;
  onRestoreName: (name: string) => void;
  onResetName: () => void;
  onPhotoClick: (photoPath: string) => void;
  onPhotoRemove: () => void;
}

export function ProfileDialog({
  chatName,
  originalName,
  profileName,
  profileNotes,
  profilePhone,
  photoPath,
  nameHistory,
  onNameChange,
  onNotesChange,
  onPhoneChange,
  onSave,
  onCancel,
  onPhotoUpload,
  onRestoreName,
  onResetName,
  onPhotoClick,
  onPhotoRemove,
}: ProfileDialogProps) {
  const [showHistory, setShowHistory] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [photoSrc, setPhotoSrc] = useState<string | null>(null);

  // Parse current phone for editing
  const parsed = parsePhoneNumber(profilePhone);
  const [selectedOption, setSelectedOption] = useState(parsed.optionId);
  const [localNumber, setLocalNumber] = useState(parsed.localNumber);
  const prevEditingRef = useRef(isEditing);

  // Only sync from profilePhone when entering edit mode, not on every keystroke
  useEffect(() => {
    const wasEditing = prevEditingRef.current;
    prevEditingRef.current = isEditing;

    // Only sync when transitioning from view to edit mode
    if (isEditing && !wasEditing) {
      const p = parsePhoneNumber(profilePhone);
      setSelectedOption(p.optionId);
      setLocalNumber(p.localNumber);
    }
  }, [isEditing, profilePhone]);

  // Load profile photo as base64 when photoPath changes
  useEffect(() => {
    if (!photoPath) {
      setPhotoSrc(null);
      return;
    }
    invoke<string>("read_file_as_base64", { path: photoPath })
      .then(setPhotoSrc)
      .catch(() => setPhotoSrc(null));
  }, [photoPath]);

  function formatHistoryDate(raw: string): string {
    const d = new Date(raw);
    return isNaN(d.getTime()) ? raw : d.toLocaleString();
  }

  function handleSave() {
    onSave();
    setIsEditing(false);
  }

  function handleCancel() {
    setIsEditing(false);
    onCancel();
  }

  return (
    <div className="dialog-overlay">
      <div className="dialog-content profile-dialog">
        <h3>{isEditing ? "Edit Profile" : "Profile"}</h3>
        <p className="profile-chat-name">{chatName}</p>
        <div className="profile-photo-section">
          {photoSrc ? (
            <img src={photoSrc} alt="Profile" className="profile-photo" onClick={() => photoSrc && onPhotoClick(photoSrc)} style={{ cursor: 'pointer' }} />
          ) : (
            <div className="profile-photo-placeholder">
              <svg viewBox="0 0 100 100" className="profile-icon">
                <circle cx="50" cy="30" r="22" fill="#e6b800"/>
                <path d="M15,85 Q15,55 50,55 Q85,55 85,85 L85,95 L15,95 Z" fill="#e6b800"/>
              </svg>
            </div>
          )}
          {isEditing && (
            <button type="button" className="upload-photo-btn" onClick={onPhotoUpload}>
              📷 Upload Photo
            </button>
          )}
          {isEditing && photoPath && (
            <button type="button" className="upload-photo-btn remove-photo-btn" onClick={onPhotoRemove}>
              🗑️ Remove Photo
            </button>
          )}
        </div>
        <div className="profile-form">
          <label>Display Name</label>
          {isEditing ? (
            <>
              <div className="profile-name-row">
                <input
                  type="text"
                  value={profileName}
                  onChange={(e) => onNameChange(e.target.value)}
                  placeholder="Enter display name"
                />
                <button
                  type="button"
                  className="reset-name-btn"
                  title={`Reset to original: "${originalName}"`}
                  onClick={onResetName}
                >
                  ↺
                </button>
              </div>
              <span className="profile-original-name">Original: {originalName}</span>
            </>
          ) : (
            <div className="profile-view-field">
              <span className="profile-view-value">{profileName || originalName}</span>
            </div>
          )}
          <label>Notes</label>
          {isEditing ? (
            <textarea
              value={profileNotes}
              onChange={(e) => onNotesChange(e.target.value)}
              placeholder="Add notes about this chat..."
              rows={4}
            />
          ) : (
            <div className="profile-view-field">
              <span className="profile-view-value profile-view-notes">{profileNotes || "No notes added"}</span>
            </div>
          )}
          {isEditing ? (
            <div className="profile-phone-row">
              <select
                className="profile-phone-select"
                value={selectedOption}
                onChange={(e) => {
                  const newOption = e.target.value;
                  setSelectedOption(newOption);
                  onPhoneChange(formatPhoneNumber(newOption, localNumber));
                }}
              >
                {phoneOptions.map(opt => (
                  <option key={opt.id} value={opt.id} title={`${opt.name} — ${opt.label}`}>
                    {opt.code} {opt.prefix || ""}
                  </option>
                ))}
              </select>
              <div className="profile-phone-input-col">
                <label>Phone Number</label>
                <input
                  type="tel"
                  className="profile-phone-input"
                  value={localNumber}
                  onChange={(e) => {
                    const validated = validatePhoneInput(e.target.value);
                    setLocalNumber(validated);
                    onPhoneChange(formatPhoneNumber(selectedOption, validated));
                  }}
                  placeholder={getPlaceholder(selectedOption)}
                />
              </div>
            </div>
          ) : (
            <div className="profile-view-field">
              <span className="profile-view-value">{profilePhone || "No phone number added"}</span>
            </div>
          )}
        </div>
        {nameHistory.length > 0 && (
          <div className="name-history-section">
            <button
              type="button"
              className="name-history-toggle"
              onClick={() => setShowHistory(v => !v)}
            >
              {showHistory ? "▾" : "▸"} Name history ({nameHistory.length})
            </button>
            {showHistory && (
              <div className="name-history-list">
                {nameHistory.map(entry => (
                  <div key={entry.id} className="name-history-item">
                    <div className="name-history-item-info">
                      <span className="name-history-name">{entry.name}</span>
                      <span className="name-history-date">{formatHistoryDate(entry.changed_at)}</span>
                    </div>
                    {isEditing && entry.name !== "[reset to original]" && (
                      <button
                        type="button"
                        className="name-history-restore"
                        onClick={() => onRestoreName(entry.name)}
                      >
                        Restore
                      </button>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
        <div className="dialog-buttons">
          {isEditing ? (
            <>
              <button type="button" onClick={handleCancel}>
                Cancel
              </button>
              <button type="button" onClick={handleSave} className="primary">
                Save
              </button>
            </>
          ) : (
            <>
              <button type="button" onClick={handleCancel}>
                Close
              </button>
              <button type="button" onClick={() => setIsEditing(true)} className="primary">
                Edit
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
