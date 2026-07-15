import type { AutoLinkEvent } from "../../types";

interface AutoLinkReviewDialogProps {
  events: AutoLinkEvent[];
  onUnlink: (contactId: string) => void;
  onClose: () => void;
}

export function AutoLinkReviewDialog({ events, onUnlink, onClose }: AutoLinkReviewDialogProps) {
  return (
    <div className="dialog-overlay">
      <div className="dialog-content profile-dialog">
        <h3>Auto-Linked Contacts</h3>
        <p className="profile-chat-name">
          {events.length} {events.length === 1 ? "person was" : "people were"} automatically linked to a matching chat.
          If any of these are wrong, unlink them below.
        </p>
        <div className="name-history-list">
          {events.map(event => (
            <div key={`${event.contact_id}-${event.chat_id}`} className="name-history-item">
              <div className="name-history-item-info">
                <span className="name-history-name">{event.contact_name} → {event.chat_name}</span>
                <span className="name-history-date">
                  {event.via_group ? `Found via "${event.via_group}"` : "Matched to an already-known contact"}
                </span>
              </div>
              <button
                type="button"
                className="name-history-restore"
                onClick={() => onUnlink(event.contact_id)}
              >
                Unlink
              </button>
            </div>
          ))}
        </div>
        <div className="dialog-buttons">
          <button type="button" onClick={onClose} className="primary">
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
