import { Metadata } from "@grpc/grpc-js";
import { SignJWT } from "jose";
import { JWT_SECRET } from "$env/static/private";

const secret = new TextEncoder().encode(JWT_SECRET);

export async function createMetadata(id: string): Promise<Metadata> {
    const metadata = new Metadata();
    const oauthToken = await new SignJWT({ id })
        .setProtectedHeader({ alg: "HS256" })
        .setExpirationTime("1h")
        .sign(secret);
    metadata.set("x-authorization", `bearer ${oauthToken}`);
    return metadata;
}
