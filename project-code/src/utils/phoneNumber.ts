/**
 * Phone number utilities for international phone number handling
 */

export interface PhoneOption {
  id: string;
  code: string;
  name: string;
  dial: string;
  prefix: string;
  label: string;
  flag: string;
}

export const phoneOptions: PhoneOption[] = [
  { id: "NL-06", code: "NL", name: "Netherlands", dial: "+31", prefix: "06", label: "06 (Mobile)", flag: "🇳🇱" },
  { id: "NL-07", code: "NL", name: "Netherlands", dial: "+31", prefix: "07", label: "07 (Mobile/Special)", flag: "🇳🇱" },
  { id: "BE-04", code: "BE", name: "Belgium", dial: "+32", prefix: "04", label: "04 (Mobile)", flag: "🇧🇪" },
  { id: "DE-015", code: "DE", name: "Germany", dial: "+49", prefix: "015", label: "015 (Mobile)", flag: "🇩🇪" },
  { id: "DE-016", code: "DE", name: "Germany", dial: "+49", prefix: "016", label: "016 (Mobile)", flag: "🇩🇪" },
  { id: "DE-017", code: "DE", name: "Germany", dial: "+49", prefix: "017", label: "017 (Mobile)", flag: "🇩🇪" },
  { id: "FR-06", code: "FR", name: "France", dial: "+33", prefix: "06", label: "06 (Mobile)", flag: "🇫🇷" },
  { id: "FR-07", code: "FR", name: "France", dial: "+33", prefix: "07", label: "07 (Mobile)", flag: "🇫🇷" },
  { id: "GB-07", code: "GB", name: "United Kingdom", dial: "+44", prefix: "07", label: "07 (Mobile)", flag: "🇬🇧" },
  { id: "US", code: "US", name: "United States", dial: "+1", prefix: "", label: "No prefix", flag: "🇺🇸" },
  { id: "ES-6", code: "ES", name: "Spain", dial: "+34", prefix: "6", label: "6 (Mobile)", flag: "🇪🇸" },
  { id: "ES-7", code: "ES", name: "Spain", dial: "+34", prefix: "7", label: "7 (Mobile)", flag: "🇪🇸" },
  { id: "IT-3", code: "IT", name: "Italy", dial: "+39", prefix: "3", label: "3 (Mobile)", flag: "🇮🇹" },
];

export function parsePhoneNumber(fullNumber: string): { optionId: string; localNumber: string } {
  if (!fullNumber) return { optionId: "NL-06", localNumber: "" };

  // Find matching option by dial code and prefix
  const sorted = [...phoneOptions].sort((a, b) => b.dial.length - a.dial.length);

  for (const opt of sorted) {
    if (fullNumber.startsWith(opt.dial)) {
      let local = fullNumber.slice(opt.dial.length).trim().replace(/^\s+/, "");
      const prefixClean = opt.prefix.replace(/^0/, "");
      const localClean = local.replace(/^0+/, "");

      if (prefixClean && localClean.startsWith(prefixClean)) {
        return { optionId: opt.id, localNumber: localClean.slice(prefixClean.length) };
      }
    }
  }

  // Default to NL 06 if no match
  return { optionId: "NL-06", localNumber: fullNumber.replace(/^[0\s]+/, "") };
}

export function formatPhoneNumber(optionId: string, localNumber: string): string {
  const opt = phoneOptions.find(o => o.id === optionId) || phoneOptions[0];
  const cleanLocal = localNumber.replace(/\s+/g, "").replace(/-/g, "");
  // Remove leading zero if present (for international format)
  const cleanNoLeadingZero = cleanLocal.replace(/^0+/, "");
  return `${opt.dial} ${cleanNoLeadingZero}`;
}

export function getPlaceholder(optionId: string): string {
  const opt = phoneOptions.find(o => o.id === optionId) || phoneOptions[0];
  // Show example without leading zero for international format
  const examplePrefix = opt.prefix ? opt.prefix.replace(/^0/, "") : "";
  return examplePrefix ? `${examplePrefix}12345678` : "1234567890";
}

export function validatePhoneInput(input: string): string {
  // Only allow digits, spaces, and dashes
  return input.replace(/[^0-9\s-]/g, "");
}
