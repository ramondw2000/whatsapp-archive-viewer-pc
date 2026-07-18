interface GroupAvatarProps {
  participants: string[];
  size?: "normal" | "large" | "dialog";
}

export function GroupAvatar({ participants, size = "normal" }: GroupAvatarProps) {
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
