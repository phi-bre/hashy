import type { CollectionEntry } from "astro:content";

const normalize = (value: string): string =>
	value
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, "-")
		.replace(/^-+|-+$/g, "");

export const hashcodeSlug = (entry: CollectionEntry<"hashcodes">): string => {
	const parts = [entry.data.year, entry.data.round]
		.map((part) => normalize(part))
		.filter(Boolean);
	const slug = parts.join("-");
	if (slug.length > 0) {
		return slug.replace(/-+/g, "-");
	}

	return normalize(entry.id.replaceAll("/", "-"));
};
