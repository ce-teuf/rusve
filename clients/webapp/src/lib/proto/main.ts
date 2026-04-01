import type * as grpc from '@grpc/grpc-js';
import type { EnumTypeDefinition, MessageTypeDefinition } from '@grpc/proto-loader';

import type { NotesServiceClient as _proto_NotesServiceClient, NotesServiceDefinition as _proto_NotesServiceDefinition } from './proto/NotesService';
import type { ScraperServiceClient as _proto_ScraperServiceClient, ScraperServiceDefinition as _proto_ScraperServiceDefinition } from './proto/ScraperService';
import type { UsersServiceClient as _proto_UsersServiceClient, UsersServiceDefinition as _proto_UsersServiceDefinition } from './proto/UsersService';
import type { UtilsServiceClient as _proto_UtilsServiceClient, UtilsServiceDefinition as _proto_UtilsServiceDefinition } from './proto/UtilsService';

type SubtypeConstructor<Constructor extends new (...args: any) => any, Subtype> = {
  new(...args: ConstructorParameters<Constructor>): Subtype;
};

export interface ProtoGrpcType {
  proto: {
    AuthResponse: MessageTypeDefinition
    Count: MessageTypeDefinition
    Email: MessageTypeDefinition
    Empty: MessageTypeDefinition
    File: MessageTypeDefinition
    FileTarget: EnumTypeDefinition
    Id: MessageTypeDefinition
    Item: MessageTypeDefinition
    ItemFilter: MessageTypeDefinition
    Job: MessageTypeDefinition
    JobResponse: MessageTypeDefinition
    Note: MessageTypeDefinition
    NoteResponse: MessageTypeDefinition
    NotesService: SubtypeConstructor<typeof grpc.Client, _proto_NotesServiceClient> & { service: _proto_NotesServiceDefinition }
    Page: MessageTypeDefinition
    Profile: MessageTypeDefinition
    ScraperService: SubtypeConstructor<typeof grpc.Client, _proto_ScraperServiceClient> & { service: _proto_ScraperServiceDefinition }
    Source: MessageTypeDefinition
    StripeUrlResponse: MessageTypeDefinition
    User: MessageTypeDefinition
    UserRole: EnumTypeDefinition
    UsersService: SubtypeConstructor<typeof grpc.Client, _proto_UsersServiceClient> & { service: _proto_UsersServiceDefinition }
    UtilsService: SubtypeConstructor<typeof grpc.Client, _proto_UtilsServiceClient> & { service: _proto_UtilsServiceDefinition }
  }
}

