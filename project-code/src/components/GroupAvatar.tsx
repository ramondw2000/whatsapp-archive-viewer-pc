interface GroupAvatarProps {
  participants: string[];
  size?: "normal" | "large" | "dialog";
}

export function GroupAvatar({ participants, size = "normal" }: GroupAvatarProps) {
  function getInitials(name: string): string {
    return name.split(" ").map(n => n[0]).join("").toUpperCase().slice(0, 2);
  }

  const slots = participants.slice(0, 4);
  const colors = ["#128c7e", "#075e54", "#00a884", "#005c4b"];

  return (
    <div className={`group-avatar group-avatar--${size}`}>
      {slots.map((name, i) => (
        <div
          key={name}
          className="group-avatar-cell"
          style={{ background: colors[i % colors.length] }}
        >
          {getInitials(name)}
        </div>
      ))}
      {slots.length === 1 && <div className="group-avatar-cell group-avatar-cell--empty" />}
    </div>
  );
}
