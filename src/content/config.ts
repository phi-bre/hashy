import { defineCollection, z } from "astro:content";

const normalizePath = (value: string): string => value.replace(/^\.\/?/u, "");

const globToFiles = async (glob: Record<string, () => Promise<unknown>>) =>
	Promise.all(
		Object.entries(glob).map(async ([file, resolver]) => {
			const resolved = await resolver();
			const url =
				typeof resolved === "string"
					? resolved
					: (resolved as { default: string }).default;
			return {
				path: normalizePath(file),
				name: file.split("/").at(-1) || "",
				url,
			};
		}),
	);

export const collections = {
	hashcodes: defineCollection({
		type: "content_layer",
		loader: async () => {
			const pdfGlob = import.meta.glob("./hashcodes/**/*.pdf", {
				query: "?url",
			});
			const metadataGlob = import.meta.glob("./hashcodes/**/*.json", {
				eager: true,
				import: "default",
			});
			const inputsGlob = import.meta.glob(
				["./hashcodes/**/*.in", "./hashcodes/**/*.in/*.txt"],
				{ query: "?url" },
			);

			const allPdfFiles = await globToFiles(pdfGlob);
			const allInputFiles = await globToFiles(inputsGlob);

			const pdfByPath = new Map(
				allPdfFiles.map((file) => [
					file.path,
					{ name: file.name, url: file.url },
				]),
			);

			return Object.entries(metadataGlob).map(([jsonPath, metadata]) => {
				const {
					year,
					round,
					title,
					description,
					scoring: scoringMetadata,
				} = metadata as {
					year: string;
					round: string;
					title: string;
					description: string;
					scoring?: { enabled?: boolean };
				};

				const normalizedJsonPath = normalizePath(jsonPath);
				const pathSegments = normalizedJsonPath.split("/");
				const type = "hashcodes";
				const id = `${type}-${year}-${round}`;
				const folder = pathSegments.slice(0, -1).join("/");
				const baseName = pathSegments.at(-1)?.replace(/\.json$/u, "") ?? "";

				const pdfPath = `${folder}/${baseName}.pdf`;
				const pdf = pdfByPath.get(pdfPath);
				if (!pdf) {
					throw new Error(`Missing PDF for metadata ${jsonPath}`);
				}

				const inputPrefix = `${folder}/${baseName}.in`;
				const inputs = allInputFiles
					.filter((file) => file.path.startsWith(inputPrefix))
					.map((file) => ({ name: file.name, url: file.url }));

				const scoring = {
					enabled: Boolean(scoringMetadata?.enabled),
				};

				return {
					id,
					type,
					year,
					round,
					title,
					description,
					pdf,
					inputs,
					scoring,
				};
			});
		},
		schema: ({ image }) =>
			z.object({
				type: z.string().default("hashcodes"),
				year: z.string(),
				round: z.string(),
				title: z.string(),
				description: z.string(),
				// cover: image(),
				pdf: z.object({ name: z.string(), url: z.string() }).optional(),
				inputs: z
					.object({ name: z.string(), url: z.string() })
					.array()
					.optional(),
				scoring: z
					.object({
						enabled: z.boolean(),
					})
					.default({ enabled: false }),
			}),
	}),
};
