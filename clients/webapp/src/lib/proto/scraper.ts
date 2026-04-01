import type * as grpc from '@grpc/grpc-js';
import type { MessageTypeDefinition } from '@grpc/proto-loader';


type SubtypeConstructor<Constructor extends new (...args: any) => any, Subtype> = {
  new(...args: ConstructorParameters<Constructor>): Subtype;
};

export interface ProtoGrpcType {
  proto: {
    Item: MessageTypeDefinition
    ItemFilter: MessageTypeDefinition
    Job: MessageTypeDefinition
    JobResponse: MessageTypeDefinition
    Source: MessageTypeDefinition
  }
}

