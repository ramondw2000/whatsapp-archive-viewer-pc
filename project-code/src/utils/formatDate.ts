/**
 * Date formatting utilities for WhatsApp timestamps
 */

export function formatDate(timestamp: string): string {
  // Extract raw date string from various timestamp formats
  const slashFull = timestamp.match(/(\d{1,2}\/\d{1,2}\/\d{4})/);
  const slashShort = timestamp.match(/(\d{1,2}\/\d{1,2}\/\d{2})(?!\d)/);
  const dashFull = timestamp.match(/(\d{2}-\d{2}-\d{4})/);
  const dotFull = timestamp.match(/(\d{2}\.\d{2}\.\d{4})/);

  let date: Date | null = null;

  if (slashFull) {
    const [d, m, y] = slashFull[1].split("/");
    date = new Date(Number(y), Number(m) - 1, Number(d));
  } else if (slashShort) {
    const [d, m, y] = slashShort[1].split("/");
    date = new Date(2000 + Number(y), Number(m) - 1, Number(d));
  } else if (dashFull) {
    const [d, m, y] = dashFull[1].split("-");
    date = new Date(Number(y), Number(m) - 1, Number(d));
  } else if (dotFull) {
    const [d, m, y] = dotFull[1].split(".");
    date = new Date(Number(y), Number(m) - 1, Number(d));
  }

  if (!date || isNaN(date.getTime())) return timestamp;

  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(today.getDate() - 1);

  const sameDay = (a: Date, b: Date) =>
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate();

  if (sameDay(date, today)) return "Today";
  if (sameDay(date, yesterday)) return "Yesterday";

  return date.toLocaleDateString(undefined, { day: "numeric", month: "long", year: "numeric" });
}
