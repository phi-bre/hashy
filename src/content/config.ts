import { defineCollection, z } from "astro:content";
import { Document } from "mupdf";

export const collections = {
	hashcodes: defineCollection({
		type: "content_layer",
		loader: async () => {
			const pdfGlob = import.meta.glob("./hashcodes/**/*.pdf");
			const inputsGlob = import.meta.glob([
				"./hashcodes/**/*.in",
				"./hashcodes/**/*.in/*.txt",
			]);

			return Object.keys(pdfGlob).map((file) => {
				const [dot, type, year, round] = file.split("/");
				const folder = [dot, type, year, round].join("/");
				const inputs = Object.keys(inputsGlob).filter((file) =>
					file.startsWith(folder),
				);

				const pdf = Document.openDocument("src/content/" + file);
				const page = JSON.parse(pdf.loadPage(0).toStructuredText().asJSON());
				const lines = page.blocks
					.flatMap((block) => block.lines)
					.map((line) => line.text.trim());
				const [title] = lines
					.filter((text) => text.length > 3) // Filter things like "EN"
					.filter((text) => !/\d\d\d\d/.test(text));

				return {
					id: `${type}-${year}-${round}`,
					type,
					year,
					round,
					title,
					cover: "",
					pdf: file,
					inputs,
				};
			});
		},
		schema: ({ image }) =>
			z.object({
				type: z.string().default("hashcodes"),
				year: z.string(),
				round: z.string(),
				title: z.string(),
				// description: z.string(),
				cover: image(),
				pdf: z.string(),
				inputs: z.array(z.string()),
			}),
	}),
};
