import { XMLParser } from "fast-xml-parser";

const alwaysArrayTags = [
  "MaterieelDeel",
  "LogischeRit",
  "LogischeRitDeel",
  "Wijziging",
];

export const setupParser = () =>
  new XMLParser({
    removeNSPrefix: true,
    ignoreAttributes: false,
    attributesGroupName: "attr",
    attributeNamePrefix: "",
    textNodeName: "text",
    parseTagValue: false,
    parseAttributeValue: false,
    isArray: (tagName, _jp, _leaf, isAttribute) =>
      !isAttribute && alwaysArrayTags.includes(tagName),
  });
