import protoLoader from "@grpc/proto-loader";
import { credentials, loadPackageDefinition } from "@grpc/grpc-js";
import { USERS_URI, NOTES_URI, UTILS_URI, GRPC_SSL } from "$env/static/private";
import type { ProtoGrpcType } from "$lib/proto/main";

export const packageDefinition = protoLoader.loadSync("./src/lib/proto/main.proto", {
    keepCase: true,
    longs: String,
    defaults: true,
    oneofs: true,
});

const proto = loadPackageDefinition(packageDefinition) as unknown as ProtoGrpcType;

const cr = GRPC_SSL === "true" ? credentials.createSsl() : credentials.createInsecure();

export const usersService = new proto.proto.UsersService(USERS_URI, cr);
export const notesService = new proto.proto.NotesService(NOTES_URI, cr);
export const utilsService = new proto.proto.UtilsService(UTILS_URI, cr);
