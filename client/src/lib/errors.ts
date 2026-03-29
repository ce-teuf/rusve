type FieldError = { field: string; tag: string };

const errorMessages: Record<string, string> = {
    req: "Field required. Length: 3-1000 characters.",
    required: "This field is required",
    min: "A minimum of 3 characters is required",
    max: "A maximum of 1000 characters is allowed",
    uuid: "Please enter a valid UUID",
    email: "Please enter a valid email address",
    url: "Please enter a valid URL",
    number: "Please enter a valid number",
    numeric: "Please enter a valid number",
    gt: "Please enter a number greater than 0",
};

export function extractError(fields: FieldError[] | undefined, field: string): string {
    if (!fields) return "";
    for (const { field: key, tag: value } of fields) {
        if (key.toLowerCase() === field.toLowerCase()) {
            return errorMessages[value] ?? "Unknown error";
        }
    }
    return "";
}
