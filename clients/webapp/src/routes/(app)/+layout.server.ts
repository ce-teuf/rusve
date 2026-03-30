import type { LayoutServerLoad } from "./$types";

export const load: LayoutServerLoad = ({ locals }) => {
    return {
        email: locals.user.email,
        avatar: locals.user.avatar,
    };
};
