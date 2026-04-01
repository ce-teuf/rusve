// Original file: main.proto

import type * as grpc from '@grpc/grpc-js'
import type { MethodDefinition } from '@grpc/proto-loader'
import type { Count as _proto_Count, Count__Output as _proto_Count__Output } from '../proto/Count';
import type { Empty as _proto_Empty, Empty__Output as _proto_Empty__Output } from '../proto/Empty';
import type { Id as _proto_Id, Id__Output as _proto_Id__Output } from '../proto/Id';
import type { Item as _proto_Item, Item__Output as _proto_Item__Output } from '../proto/Item';
import type { ItemFilter as _proto_ItemFilter, ItemFilter__Output as _proto_ItemFilter__Output } from '../proto/ItemFilter';
import type { Job as _proto_Job, Job__Output as _proto_Job__Output } from '../proto/Job';
import type { JobResponse as _proto_JobResponse, JobResponse__Output as _proto_JobResponse__Output } from '../proto/JobResponse';
import type { Page as _proto_Page, Page__Output as _proto_Page__Output } from '../proto/Page';
import type { Source as _proto_Source, Source__Output as _proto_Source__Output } from '../proto/Source';

export interface ScraperServiceClient extends grpc.Client {
  ApproveAllValid(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  ApproveAllValid(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  ApproveAllValid(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  ApproveAllValid(argument: _proto_Id, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  approveAllValid(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  approveAllValid(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  approveAllValid(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  approveAllValid(argument: _proto_Id, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  
  ApproveItem(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  ApproveItem(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  ApproveItem(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  ApproveItem(argument: _proto_Id, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  approveItem(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  approveItem(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  approveItem(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  approveItem(argument: _proto_Id, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  
  CreateSource(argument: _proto_Source, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  CreateSource(argument: _proto_Source, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  CreateSource(argument: _proto_Source, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  CreateSource(argument: _proto_Source, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  createSource(argument: _proto_Source, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  createSource(argument: _proto_Source, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  createSource(argument: _proto_Source, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  createSource(argument: _proto_Source, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  
  DeleteSource(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  DeleteSource(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  DeleteSource(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  DeleteSource(argument: _proto_Id, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  deleteSource(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  deleteSource(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  deleteSource(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  deleteSource(argument: _proto_Id, callback: grpc.requestCallback<_proto_Empty__Output>): grpc.ClientUnaryCall;
  
  GetJobById(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  GetJobById(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  GetJobById(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  GetJobById(argument: _proto_Id, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  getJobById(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  getJobById(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  getJobById(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  getJobById(argument: _proto_Id, callback: grpc.requestCallback<_proto_Job__Output>): grpc.ClientUnaryCall;
  
  GetSource(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  GetSource(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  GetSource(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  GetSource(argument: _proto_Id, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  getSource(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  getSource(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  getSource(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  getSource(argument: _proto_Id, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  
  ListItems(argument: _proto_ItemFilter, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Item__Output>;
  ListItems(argument: _proto_ItemFilter, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Item__Output>;
  listItems(argument: _proto_ItemFilter, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Item__Output>;
  listItems(argument: _proto_ItemFilter, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Item__Output>;
  
  ListJobs(argument: _proto_Page, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_JobResponse__Output>;
  ListJobs(argument: _proto_Page, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_JobResponse__Output>;
  listJobs(argument: _proto_Page, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_JobResponse__Output>;
  listJobs(argument: _proto_Page, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_JobResponse__Output>;
  
  ListSources(argument: _proto_Empty, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Source__Output>;
  ListSources(argument: _proto_Empty, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Source__Output>;
  listSources(argument: _proto_Empty, metadata: grpc.Metadata, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Source__Output>;
  listSources(argument: _proto_Empty, options?: grpc.CallOptions): grpc.ClientReadableStream<_proto_Source__Output>;
  
  PushApproved(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  PushApproved(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  PushApproved(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  PushApproved(argument: _proto_Id, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  pushApproved(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  pushApproved(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  pushApproved(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  pushApproved(argument: _proto_Id, callback: grpc.requestCallback<_proto_Count__Output>): grpc.ClientUnaryCall;
  
  RejectItem(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  RejectItem(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  RejectItem(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  RejectItem(argument: _proto_Id, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  rejectItem(argument: _proto_Id, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  rejectItem(argument: _proto_Id, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  rejectItem(argument: _proto_Id, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  rejectItem(argument: _proto_Id, callback: grpc.requestCallback<_proto_Item__Output>): grpc.ClientUnaryCall;
  
  UpdateSource(argument: _proto_Source, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  UpdateSource(argument: _proto_Source, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  UpdateSource(argument: _proto_Source, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  UpdateSource(argument: _proto_Source, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  updateSource(argument: _proto_Source, metadata: grpc.Metadata, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  updateSource(argument: _proto_Source, metadata: grpc.Metadata, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  updateSource(argument: _proto_Source, options: grpc.CallOptions, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  updateSource(argument: _proto_Source, callback: grpc.requestCallback<_proto_Source__Output>): grpc.ClientUnaryCall;
  
}

export interface ScraperServiceHandlers extends grpc.UntypedServiceImplementation {
  ApproveAllValid: grpc.handleUnaryCall<_proto_Id__Output, _proto_Count>;
  
  ApproveItem: grpc.handleUnaryCall<_proto_Id__Output, _proto_Item>;
  
  CreateSource: grpc.handleUnaryCall<_proto_Source__Output, _proto_Source>;
  
  DeleteSource: grpc.handleUnaryCall<_proto_Id__Output, _proto_Empty>;
  
  GetJobById: grpc.handleUnaryCall<_proto_Id__Output, _proto_Job>;
  
  GetSource: grpc.handleUnaryCall<_proto_Id__Output, _proto_Source>;
  
  ListItems: grpc.handleServerStreamingCall<_proto_ItemFilter__Output, _proto_Item>;
  
  ListJobs: grpc.handleServerStreamingCall<_proto_Page__Output, _proto_JobResponse>;
  
  ListSources: grpc.handleServerStreamingCall<_proto_Empty__Output, _proto_Source>;
  
  PushApproved: grpc.handleUnaryCall<_proto_Id__Output, _proto_Count>;
  
  RejectItem: grpc.handleUnaryCall<_proto_Id__Output, _proto_Item>;
  
  UpdateSource: grpc.handleUnaryCall<_proto_Source__Output, _proto_Source>;
  
}

export interface ScraperServiceDefinition extends grpc.ServiceDefinition {
  ApproveAllValid: MethodDefinition<_proto_Id, _proto_Count, _proto_Id__Output, _proto_Count__Output>
  ApproveItem: MethodDefinition<_proto_Id, _proto_Item, _proto_Id__Output, _proto_Item__Output>
  CreateSource: MethodDefinition<_proto_Source, _proto_Source, _proto_Source__Output, _proto_Source__Output>
  DeleteSource: MethodDefinition<_proto_Id, _proto_Empty, _proto_Id__Output, _proto_Empty__Output>
  GetJobById: MethodDefinition<_proto_Id, _proto_Job, _proto_Id__Output, _proto_Job__Output>
  GetSource: MethodDefinition<_proto_Id, _proto_Source, _proto_Id__Output, _proto_Source__Output>
  ListItems: MethodDefinition<_proto_ItemFilter, _proto_Item, _proto_ItemFilter__Output, _proto_Item__Output>
  ListJobs: MethodDefinition<_proto_Page, _proto_JobResponse, _proto_Page__Output, _proto_JobResponse__Output>
  ListSources: MethodDefinition<_proto_Empty, _proto_Source, _proto_Empty__Output, _proto_Source__Output>
  PushApproved: MethodDefinition<_proto_Id, _proto_Count, _proto_Id__Output, _proto_Count__Output>
  RejectItem: MethodDefinition<_proto_Id, _proto_Item, _proto_Id__Output, _proto_Item__Output>
  UpdateSource: MethodDefinition<_proto_Source, _proto_Source, _proto_Source__Output, _proto_Source__Output>
}
