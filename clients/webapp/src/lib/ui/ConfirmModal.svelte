<script lang="ts">
    import { enhance } from "$app/forms";
    import Button from "$lib/form/Button.svelte";
    import Modal from "./Modal.svelte";
    import { toast } from "./toast";

    interface Props {
        open: boolean;
        id: string;
    }

    let { open = $bindable(), id }: Props = $props();
</script>

<Modal
    bind:open
    title="Confirm"
    description="Are you sure you want to delete this item? This action cannot be undone."
>
    <form
        action="?/delete"
        method="post"
        use:enhance={() => {
            return async ({ result, update }) => {
                if (result.type === "success") toast.warning("Success", "Item deleted");
                await update();
                history.back();
            };
        }}
    >
        <input type="hidden" name="id" value={id} />
        <Button variant="danger" class="w-full">Delete</Button>
    </form>
</Modal>
