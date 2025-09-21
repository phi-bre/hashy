// @ts-check
import { defineConfig } from "astro/config";
import fulldevUi from "fulldev-ui";

// https://astro.build/config
export default defineConfig({
	integrations: [fulldevUi({})],
	experimental: {
		contentLayer: true,
	},
});
