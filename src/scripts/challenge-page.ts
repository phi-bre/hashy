import "pdfjs-dist/web/pdf_viewer.css";
import pdfWorker from "pdfjs-dist/build/pdf.worker?url";
import { AnnotationMode, GlobalWorkerOptions, getDocument } from "pdfjs-dist";
import { EventBus, PDFLinkService, PDFViewer } from "pdfjs-dist/web/pdf_viewer";

const TEXT_LAYER_MODE_ENABLE = 1;
const ANNOTATION_MODE_ENABLE =
        typeof AnnotationMode !== "undefined" && typeof AnnotationMode.ENABLE === "number"
                ? AnnotationMode.ENABLE
                : 1;

type PdfViewerContainer = HTMLElement & {
        dataset: DOMStringMap & {
                src?: string;
                pdfInitialized?: string;
        };
};

type DetailSection = {
        key: string;
        label: string;
        text: string;
        multiline: boolean;
        classes: string;
};

type MetaSection = {
        label: string;
        value: string;
};

const escapeHtml = (value: unknown): string =>
        String(value)
                .replace(/&/g, "&amp;")
                .replace(/</g, "&lt;")
                .replace(/>/g, "&gt;")
                .replace(/"/g, "&quot;")
                .replace(/'/g, "&#39;");

const formatLabel = (value: unknown): string =>
        String(value)
                .split(/[_\s-]+/u)
                .map((part) => part.trim())
                .filter(Boolean)
                .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
                .join(" ");

const formatDetailValue = (value: unknown): { text: string; multiline: boolean; monospace: boolean } => {
        if (typeof value === "number" && Number.isFinite(value)) {
                return { text: value.toLocaleString(), multiline: false, monospace: false };
        }
        if (value && typeof value === "object") {
                try {
                        return {
                                text: JSON.stringify(value, null, 2),
                                multiline: true,
                                monospace: true,
                        };
                } catch (error) {
                        const stringValue = String(value);
                        return { text: stringValue, multiline: /\n/u.test(stringValue), monospace: false };
                }
        }
        const text = value == null ? "" : String(value);
        const multiline = /\n/u.test(text);
        return { text, multiline, monospace: multiline };
};

const buildSuccess = (result: Record<string, unknown>): string => {
        const rawScore = typeof result.score === "number" ? result.score : Number(result.score ?? 0);
        const scoreText = Number.isFinite(rawScore)
                ? rawScore.toLocaleString()
                : escapeHtml(String(result.score ?? ""));
        const dataset = typeof result.input_file === "string" ? result.input_file : "";
        const problem = typeof result.problem === "string" ? result.problem : "";
        const details =
                result && typeof result.details === "object" && result.details !== null
                        ? Object.entries(result.details as Record<string, unknown>)
                        : [];

        const metaSections: MetaSection[] = [];
        if (dataset) {
                metaSections.push({ label: "Dataset", value: formatLabel(dataset) });
        }
        if (problem) {
                metaSections.push({ label: "Problem", value: formatLabel(problem) });
        }

        const detailSections: DetailSection[] = details
                .filter((entry): entry is [string, unknown] => Array.isArray(entry) && entry.length === 2)
                .map(([key, value]) => {
                        const { text, multiline, monospace } = formatDetailValue(value);
                        const classes = [
                                "text-sm",
                                "text-muted-foreground",
                                "rounded-xl",
                                "border",
                                "border-border/60",
                                "bg-background/60",
                                "px-4",
                                "py-3",
                                "whitespace-pre-wrap",
                        ];
                        if (monospace) {
                                classes.push("font-mono", "text-xs");
                        }
                        return {
                                key,
                                label: formatLabel(key),
                                text,
                                multiline,
                                classes: classes.join(" "),
                        };
                });

        const detailMarkup = detailSections
                .map(({ key, label, text, classes }) => {
                        const escapedText = escapeHtml(text);
                        return `
                                <li class="space-y-1" data-detail="${escapeHtml(key)}">
                                        <p class="text-xs font-semibold uppercase tracking-[0.32em] text-muted-foreground">${escapeHtml(
                                                label,
                                        )}</p>
                                        <pre class="${classes}">${escapedText}</pre>
                                </li>
                        `;
                })
                .join("");

        const metaMarkup = metaSections
                .map(
                        ({ label, value }) => `
                                <li class="flex flex-col gap-1">
                                        <p class="text-xs font-semibold uppercase tracking-[0.32em] text-muted-foreground">${escapeHtml(
                                                label,
                                        )}</p>
                                        <p class="text-sm font-medium text-foreground">${escapeHtml(value)}</p>
                                </li>
                        `,
                )
                .join("");

        const hasDetails = detailSections.length > 0;

        return `
                <div class="rounded-2xl border border-emerald-700/40 bg-emerald-600/10 p-5 text-sm text-emerald-200">
                        <div class="flex flex-wrap items-center justify-between gap-3">
                                <div>
                                        <p class="text-xs font-semibold uppercase tracking-[0.32em] text-emerald-200/80">Submission scored</p>
                                        <p class="mt-1 text-2xl font-semibold text-emerald-100">${scoreText}</p>
                                </div>
                                ${
                                        metaMarkup
                                                ? `<ul class="grid gap-3 text-left sm:grid-cols-${Math.min(2, metaSections.length)}">${metaMarkup}</ul>`
                                                : ""
                                }
                        </div>
                        ${
                                hasDetails
                                        ? `<ul class="mt-5 grid gap-4 sm:grid-cols-${Math.min(2, detailSections.length)}">${detailMarkup}</ul>`
                                        : ""
                        }
                </div>
        `;
};

const buildError = (code: unknown, message: unknown, details?: unknown): string => {
        const detailMarkup =
                details && typeof details === "object"
                        ? `<pre class="mt-4 whitespace-pre-wrap rounded-xl border border-destructive/40 bg-destructive/10 px-4 py-3 text-xs text-destructive/90">${escapeHtml(
                                JSON.stringify(details, null, 2),
                        )}</pre>`
                        : "";
        return `
                <div class="rounded-2xl border border-destructive/40 bg-destructive/10 p-5 text-sm text-destructive/90">
                        <p class="text-xs font-semibold uppercase tracking-[0.32em] text-destructive/60">${escapeHtml(code)}</p>
                        <p class="mt-1 text-base font-semibold text-destructive">${escapeHtml(message)}</p>
                        ${detailMarkup}
                </div>
        `;
};

const initializePdfViewer = async (container: PdfViewerContainer): Promise<void> => {
        if (!container || container.dataset.pdfInitialized === "true") {
                return;
        }

        const src = container.dataset.src;
        const placeholder = container.querySelector<HTMLElement>("[data-pdf-placeholder]");

        if (!src) {
                if (placeholder) {
                        placeholder.innerHTML = '<p class="text-sm text-muted-foreground">PDF unavailable.</p>';
                }
                return;
        }

        try {
                GlobalWorkerOptions.workerSrc = pdfWorker;
        } catch (error) {
                console.error("Failed to configure PDF.js worker", error);
        }

        const viewerContainer = document.createElement("div");
        viewerContainer.className = "pdfjs-viewer-container";
        viewerContainer.setAttribute("role", "presentation");
        viewerContainer.tabIndex = 0;
        viewerContainer.hidden = true;

        const viewer = document.createElement("div");
        viewer.className = "pdfViewer";
        viewerContainer.appendChild(viewer);
        container.appendChild(viewerContainer);

        const eventBus = new EventBus();
        const linkService = new PDFLinkService({ eventBus });

        let pdfDocument;
        try {
                pdfDocument = await getDocument({ url: src }).promise;
        } catch (error) {
                if (placeholder) {
                        placeholder.innerHTML = '<p class="text-sm text-muted-foreground">Unable to load PDF.</p>';
                }
                console.error("Failed to fetch PDF", error);
                viewerContainer.remove();
                return;
        }

        container.dataset.pdfInitialized = "true";

        const pdfViewer = new PDFViewer({
                container: viewerContainer,
                eventBus,
                linkService,
                textLayerMode: TEXT_LAYER_MODE_ENABLE,
                annotationMode: ANNOTATION_MODE_ENABLE,
                renderInteractiveForms: true,
                useOnlyCssZoom: true,
        });

        linkService.setViewer(pdfViewer);

        eventBus.on("pagesinit", () => {
                pdfViewer.currentScaleValue = "page-width";
        });

        linkService.setDocument(pdfDocument, null);
        pdfViewer.setDocument(pdfDocument);

        viewerContainer.hidden = false;

        if (placeholder) {
                placeholder.remove();
        }
};

const initializeScoringForms = (): void => {
        const scoringForms = document.querySelectorAll<HTMLFormElement>("[data-scoring-form]");

        scoringForms.forEach((form) => {
                const results = form.querySelector<HTMLElement>("[data-scoring-results]");
                const datasetField = form.querySelector("[name=\"dataset\"]");
                const submissionField = form.querySelector("[name=\"submission\"]");
                const submitButton = form.querySelector<HTMLButtonElement>('[type="submit"]');
                const defaultLabel = submitButton?.querySelector<HTMLElement>('[data-default-label]');
                const loadingLabel = submitButton?.querySelector<HTMLElement>('[data-loading-label]');

                const setLoading = (loading: boolean) => {
                        form.classList.toggle("pointer-events-none", loading);
                        form.classList.toggle("opacity-70", loading);
                        if (datasetField instanceof HTMLSelectElement) {
                                datasetField.disabled = loading;
                        }
                        if (submissionField instanceof HTMLInputElement) {
                                submissionField.disabled = loading;
                        }
                        if (defaultLabel && loadingLabel) {
                                if (loading) {
                                        defaultLabel.classList.add("hidden");
                                        loadingLabel.classList.remove("hidden");
                                } else {
                                        defaultLabel.classList.remove("hidden");
                                        loadingLabel.classList.add("hidden");
                                }
                        }
                };

                form.addEventListener("submit", async (event) => {
                        event.preventDefault();
                        if (!results || !datasetField || !submissionField) {
                                return;
                        }
                        if (typeof form.reportValidity === "function" && !form.reportValidity()) {
                                return;
                        }

                        if (!(datasetField instanceof HTMLSelectElement) || !(submissionField instanceof HTMLInputElement)) {
                                return;
                        }

                        const dataset = datasetField.value.trim();
                        const file = submissionField.files && submissionField.files[0] ? submissionField.files[0] : null;
                        results.innerHTML = "";

                        if (!dataset) {
                                results.innerHTML = buildError("missing-dataset", "Select an input dataset before scoring.");
                                datasetField.focus();
                                return;
                        }

                        if (!file) {
                                results.innerHTML = buildError("missing-file", "Upload a submission file before scoring.");
                                submissionField.focus();
                                return;
                        }

                        let submission = "";
                        try {
                                submission = await file.text();
                        } catch (error) {
                                results.innerHTML = buildError(
                                        "file-read-error",
                                        "We couldn't read that file. Please upload a valid text submission.",
                                );
                                submissionField.value = "";
                                submissionField.focus();
                                return;
                        }

                        if (!submission.trim()) {
                                results.innerHTML = buildError(
                                        "empty-file",
                                        "The uploaded submission file is empty. Add content before scoring.",
                                );
                                submissionField.focus();
                                return;
                        }

                        setLoading(true);
                        try {
                                const year = form.getAttribute("data-year") ?? "";
                                const round = form.getAttribute("data-round") ?? "";
                                const endpoint = `/api/hashcodes/${encodeURIComponent(year)}/${encodeURIComponent(round)}/${encodeURIComponent(
                                        dataset,
                                )}`;

                                const response = await fetch(endpoint, {
                                        method: "POST",
                                        body: submission,
                                        headers: {
                                                "Content-Type": "text/plain;charset=utf-8",
                                        },
                                });

                                let payload: unknown = null;
                                try {
                                        payload = await response.json();
                                } catch (error) {
                                        payload = null;
                                }

                                if (response.ok && payload && typeof payload === "object" && (payload as Record<string, unknown>).status === "ok") {
                                        const result = (payload as { result?: Record<string, unknown> }).result;
                                        if (result && typeof result === "object") {
                                                results.innerHTML = buildSuccess(result);
                                        } else {
                                                results.innerHTML = buildError(
                                                        "missing-result",
                                                        "The scorer returned an empty payload.",
                                                        payload,
                                                );
                                        }
                                } else {
                                        const errorPayload =
                                                payload &&
                                                typeof payload === "object" &&
                                                (payload as Record<string, unknown>).error &&
                                                typeof (payload as { error?: unknown }).error === "object"
                                                        ? ((payload as { error?: Record<string, unknown> }).error ?? undefined)
                                                        : undefined;

                                        const errorCode = errorPayload?.code ?? (response.ok ? "unknown-error" : `http-${response.status}`);
                                        const errorMessage = errorPayload?.message ?? "Unable to score the submission.";
                                        const errorDetails = errorPayload?.details ?? (response.ok ? undefined : payload);
                                        results.innerHTML = buildError(errorCode, errorMessage, errorDetails);
                                }
                        } catch (error) {
                                const message = error instanceof Error ? error.message : "Unexpected network error.";
                                results.innerHTML = buildError("network-error", message);
                        } finally {
                                setLoading(false);
                        }
                });
        });
};

const initialize = () => {
        const pdfContainers = Array.from(
                document.querySelectorAll<PdfViewerContainer>("[data-pdf-viewer][data-src]") ?? [],
        );

        if (pdfContainers.length > 0) {
                void Promise.all(pdfContainers.map((container) => initializePdfViewer(container)));
        }

        initializeScoringForms();
};

if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", initialize, { once: true });
} else {
        initialize();
}
