import { Drawer as DrawerPrimitive } from "vaul-svelte";
import Content from "./drawer-content.svelte";
import Description from "./drawer-description.svelte";
import Footer from "./drawer-footer.svelte";
import Header from "./drawer-header.svelte";
import Title from "./drawer-title.svelte";

const Root = DrawerPrimitive.Root;
const Trigger = DrawerPrimitive.Trigger;
const Portal = DrawerPrimitive.Portal;
const Close = DrawerPrimitive.Close;
const Overlay = DrawerPrimitive.Overlay;
const Handle = DrawerPrimitive.Handle;

export {
    Root,
    Trigger,
    Portal,
    Close,
    Overlay,
    Handle,
    Content,
    Description,
    Footer,
    Header,
    Title,
    Root as Drawer,
    Trigger as DrawerTrigger,
    Portal as DrawerPortal,
    Close as DrawerClose,
    Overlay as DrawerOverlay,
    Handle as DrawerHandle,
    Content as DrawerContent,
    Description as DrawerDescription,
    Footer as DrawerFooter,
    Header as DrawerHeader,
    Title as DrawerTitle,
};
