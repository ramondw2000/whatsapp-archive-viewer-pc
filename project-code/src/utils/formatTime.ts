export function formatTime(timestamp: string): string {
  const match = timestamp.match(/(\d{2}:\d{2})/);
  return match ? match[1] : timestamp;
}
