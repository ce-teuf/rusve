import { redirect } from "@sveltejs/kit";
import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = ({ locals }) => {
    if (locals.user.role !== 2) {
        redirect(302, "/dashboard");
    }
    return {
        email: locals.user.email,
        avatar: locals.user.avatar,
    };
};
