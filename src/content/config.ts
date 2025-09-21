import {defineCollection, z} from "astro:content";

export const collections = {
    hashcodes: defineCollection({
        type: "content_layer",
        loader: async () => {
            const globToFiles = (glob) => Promise.all(
                Object.entries(glob).map(async ([file, url]) => ({
                    path: file,
                    name: file.split("/").at(-1) || '',
                    url: (await url()).default,
                })),
            )

            const pdfGlob = import.meta.glob("./hashcodes/**/*.pdf", {query: "?url"});
            const metadataGlob = import.meta.glob("./hashcodes/**/*.json", {
                eager: true,
                import: "default",
            });
            const inputsGlob = import.meta.glob(
                ["./hashcodes/**/*.in", "./hashcodes/**/*.in/*.txt"],
                {query: "?url"},
            );

            const allPdfFiles = await globToFiles(pdfGlob);
            const allInputFiles = await globToFiles(inputsGlob);

            const pdfByPath = new Map(allPdfFiles.map((file) => [file.path, file]));

            return Object.entries(metadataGlob).map(([jsonPath, metadata]) => {
                const {year, round, title, description} = metadata as {
                    year: string;
                    round: string;
                    title: string;
                    description: string;
                };

                const pathSegments = jsonPath.split("/");
                const type = pathSegments.at(1) ?? "hashcodes";
                const id = `${type}-${year}-${round}`;
                const folder = pathSegments.slice(0, -1).join("/");

                const pdfPath = jsonPath.replace(/\.json$/u, ".pdf");
                const pdf = pdfByPath.get(pdfPath);
                if (!pdf) {
                    throw new Error(`Missing PDF for metadata ${jsonPath}`);
                }

                const inputs = allInputFiles.filter((file) => file.path.startsWith(folder));

                console.log({id, type, year, round, title, description, pdf, inputs})
                return {id, type, year, round, title, description, pdf, inputs};
            });
        },
        schema: ({image}) =>
            z.object({
                type: z.string().default("hashcodes"),
                year: z.string(),
                round: z.string(),
                title: z.string(),
                description: z.string(),
                // cover: image(),
                pdf: z.object({name: z.string(), url: z.string()}).optional(),
                inputs: z.object({name: z.string(), url: z.string()}).array().optional(),
            }),
    }),
};
