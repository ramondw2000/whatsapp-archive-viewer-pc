const CHAT_PREFIXES = [
  "WhatsApp-chat met ",
  "WhatsApp-gesprek met ",
  "WhatsApp Chat with ",
  "WhatsApp-Unterhaltung mit ",
];

export function stripChatPrefix(name: string): string {
  for (const prefix of CHAT_PREFIXES) {
    if (name.toLowerCase().startsWith(prefix.toLowerCase())) {
      return name.slice(prefix.length);
    }
  }
  return name;
}
