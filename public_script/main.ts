// @ts-expect-error (no types)
import ajax from "@imacrayon/alpine-ajax";
import Alpine from "alpinejs";

globalThis.Alpine = Alpine;

Alpine.plugin(ajax);

Alpine.start();
