import { Dialog as SheetPrimitive } from "bits-ui";
import Content from "./sheet-content.svelte";
import Description from "./sheet-description.svelte";
import Footer from "./sheet-footer.svelte";
import Header from "./sheet-header.svelte";
import Title from "./sheet-title.svelte";

const Root = SheetPrimitive.Root;
const Trigger = SheetPrimitive.Trigger;
const Close = SheetPrimitive.Close;
const Portal = SheetPrimitive.Portal;
const Overlay = SheetPrimitive.Overlay;

export {
    Root,
    Trigger,
    Close,
    Portal,
    Overlay,
    Content,
    Description,
    Footer,
    Header,
    Title,
};
