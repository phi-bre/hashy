import {defineCollection, z} from "astro:content";
import {Document} from "mupdf";

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
            const inputsGlob = import.meta.glob(
                ["./hashcodes/**/*.in", "./hashcodes/**/*.in/*.txt"],
                {query: "?url"},
            );
            const allPdfFiles = await globToFiles(pdfGlob);
            const allInputFiles = await globToFiles(inputsGlob)

            return allPdfFiles.map((pdf) => {
                const [dot, type, year, round] = pdf.path.split("/");
                const id = `${type}-${year}-${round}`;
                const folder = [dot, type, year, round].join("/");
                const inputs = allInputFiles.filter((file) => file.path.startsWith(folder));
                const document = Document.openDocument("src/content/" + pdf.path);
                const page = JSON.parse(
                    document.loadPage(0).toStructuredText().asJSON(),
                );
                const lines = page.blocks
                    .flatMap((block) => block.lines)
                    .map((line) => line.text.trim());
                const [title] = lines
                    .filter((text) => text.length > 3) // Filter things like "EN"
                    .filter((text) => !/\d\d\d\d/.test(text));

                return {id, type, year, round, title, pdf, inputs};
            });
        },
        schema: ({image}) =>
            z.object({
                type: z.string().default("hashcodes"),
                year: z.string(),
                round: z.string(),
                title: z.string(),
                // description: z.string(),
                // cover: image(),
                pdf: z.object({name: z.string(), url: z.string()}),
                inputs: z.object({name: z.string(), url: z.string()}).array(),
            }),
    }),
};
