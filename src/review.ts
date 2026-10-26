import type { AppraisalIvs, ImportInput } from "./native";

export interface ReviewFields {
  displayName: string;
  cp: string;
  attack: string;
  defense: string;
  hp: string;
}
export const emptyReview: ReviewFields = {
  displayName: "",
  cp: "",
  attack: "",
  defense: "",
  hp: "",
};

export function validateReview(fields: ReviewFields): ImportInput | string {
  if (
    !fields.displayName ||
    fields.displayName.trim() !== fields.displayName ||
    Array.from(fields.displayName).length > 100 ||
    /\p{Cc}/u.test(fields.displayName)
  )
    return "Enter a display name of 1–100 characters, without surrounding whitespace or control characters.";
  if (
    fields.cp !== "" &&
    (!/^[1-9][0-9]*$/.test(fields.cp) ||
      fields.cp.length > 19 ||
      BigInt(fields.cp) > 9223372036854775807n)
  )
    return "CP must be a whole number from 1 to 9223372036854775807, or empty if unknown.";
  const ivs = validateIvs(fields);
  if (typeof ivs === "string") return ivs;
  return {
    displayName: fields.displayName,
    cp: fields.cp || null,
    ivs,
    tagIds: [],
  };
}

export function validateIvs(
  fields: Pick<ReviewFields, "attack" | "defense" | "hp">,
): AppraisalIvs | null | string {
  const ivs = [fields.attack, fields.defense, fields.hp];
  const unknown = ivs.every((value) => value === "");
  if (
    !unknown &&
    !ivs.every((value) => /^[0-9]{1,2}$/.test(value) && Number(value) <= 15)
  )
    return "Leave all three appraisal IVs empty if unknown, or enter all three as whole numbers from 0 to 15.";
  return unknown
    ? null
    : {
        attack: Number(fields.attack),
        defense: Number(fields.defense),
        hp: Number(fields.hp),
      };
}
