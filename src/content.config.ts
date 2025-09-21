import { defineCollection, z } from 'astro:content';
import { glob, file } from 'astro/loaders';

const hashcodes = defineCollection({
    type: 'data',
    // Build entries by walking /public/hashcodes
    loader: async () => {
        const fs = await import('node:fs/promises');
        const path = await import('node:path');
        const root = path.join(process.cwd(), 'public', 'hashcodes');

        const walk = async (dir: string, parts: string[] = []) => {
            const names = await fs.readdir(dir);
            const out: any[] = [];
            for (const name of names) {
                const p = path.join(dir, name);
                const stat = await fs.stat(p);
                if (stat.isDirectory()) out.push(...await walk(p, [...parts, name]));
                else out.push({
                    id: [...parts, name].join('/'),
                    year: parts[0],
                    round: parts[1] ?? '',
                    file: name,
                    url: `/hashcodes/${[...parts, name].join('/')}`,
                });
            }
            return out;
        };

        return await walk(root);
    },
    schema: z.object({
        year: z.string(),
        round: z.string(),
        file: z.string(),
        url: z.string().url(),
    }),
});

export const collections = { hashcodes };
