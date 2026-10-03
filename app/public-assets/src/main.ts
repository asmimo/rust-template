// @ts-expect-error (no types)
import ajax from "@imacrayon/alpine-ajax";
import Alpine from "alpinejs";

globalThis.Alpine = Alpine;

// oxlint-disable-next-line @typescript/no-unsafe-argument
Alpine.plugin(ajax);

Alpine.start();
