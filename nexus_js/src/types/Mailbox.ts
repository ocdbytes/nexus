/* Autogenerated file. Do not edit manually. */
/* tslint:disable */
/* eslint-disable */
import type {
  BaseContract,
  BigNumberish,
  BytesLike,
  FunctionFragment,
  Result,
  Interface,
  EventFragment,
  AddressLike,
  ContractRunner,
  ContractMethod,
  Listener,
} from "ethers";
import type {
  TypedContractEvent,
  TypedDeferredTopicFilter,
  TypedEventLog,
  TypedLogDescription,
  TypedListener,
  TypedContractMethod,
} from "./common.js";

export type MailboxMessageStruct = {
  nexusAppIDFrom: BytesLike;
  nexusAppIDTo: BytesLike[];
  data: BytesLike;
  from: AddressLike;
  to: AddressLike[];
  nonce: BigNumberish;
};

export type MailboxMessageStructOutput = [
  nexusAppIDFrom: string,
  nexusAppIDTo: string[],
  data: string,
  from: string,
  to: string[],
  nonce: bigint
] & {
  nexusAppIDFrom: string;
  nexusAppIDTo: string[];
  data: string;
  from: string;
  to: string[];
  nonce: bigint;
};

export interface MailboxInterface extends Interface {
  getFunction(
    nameOrSignature:
      | "addOrUpdateWrapper"
      | "initialize"
      | "messages"
      | "nexusAppId"
      | "owner"
      | "receiveMessage"
      | "renounceOwnership"
      | "sendMessage"
      | "transferOwnership"
      | "verifiedReceipts"
      | "verifierWrappers"
  ): FunctionFragment;

  getEvent(
    nameOrSignatureOrTopic:
      | "CallbackFailed"
      | "Initialized"
      | "MailboxEvent"
      | "OwnershipTransferred"
  ): EventFragment;

  encodeFunctionData(
    functionFragment: "addOrUpdateWrapper",
    values: [BytesLike, AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "initialize",
    values?: undefined
  ): string;
  encodeFunctionData(functionFragment: "messages", values: [BytesLike]): string;
  encodeFunctionData(
    functionFragment: "nexusAppId",
    values?: undefined
  ): string;
  encodeFunctionData(functionFragment: "owner", values?: undefined): string;
  encodeFunctionData(
    functionFragment: "receiveMessage",
    values: [BigNumberish, MailboxMessageStruct, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "renounceOwnership",
    values?: undefined
  ): string;
  encodeFunctionData(
    functionFragment: "sendMessage",
    values: [BytesLike[], AddressLike[], BigNumberish, BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "transferOwnership",
    values: [AddressLike]
  ): string;
  encodeFunctionData(
    functionFragment: "verifiedReceipts",
    values: [BytesLike]
  ): string;
  encodeFunctionData(
    functionFragment: "verifierWrappers",
    values: [BytesLike]
  ): string;

  decodeFunctionResult(
    functionFragment: "addOrUpdateWrapper",
    data: BytesLike
  ): Result;
  decodeFunctionResult(functionFragment: "initialize", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "messages", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "nexusAppId", data: BytesLike): Result;
  decodeFunctionResult(functionFragment: "owner", data: BytesLike): Result;
  decodeFunctionResult(
    functionFragment: "receiveMessage",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "renounceOwnership",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "sendMessage",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "transferOwnership",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "verifiedReceipts",
    data: BytesLike
  ): Result;
  decodeFunctionResult(
    functionFragment: "verifierWrappers",
    data: BytesLike
  ): Result;
}

export namespace CallbackFailedEvent {
  export type InputTuple = [to: AddressLike, data: BytesLike];
  export type OutputTuple = [to: string, data: string];
  export interface OutputObject {
    to: string;
    data: string;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace InitializedEvent {
  export type InputTuple = [version: BigNumberish];
  export type OutputTuple = [version: bigint];
  export interface OutputObject {
    version: bigint;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace MailboxEventEvent {
  export type InputTuple = [
    nexusAppIDFrom: BytesLike,
    nexusAppIDTo: BytesLike[],
    data: BytesLike,
    from: AddressLike,
    to: AddressLike[],
    nonce: BigNumberish
  ];
  export type OutputTuple = [
    nexusAppIDFrom: string,
    nexusAppIDTo: string[],
    data: string,
    from: string,
    to: string[],
    nonce: bigint
  ];
  export interface OutputObject {
    nexusAppIDFrom: string;
    nexusAppIDTo: string[];
    data: string;
    from: string;
    to: string[];
    nonce: bigint;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export namespace OwnershipTransferredEvent {
  export type InputTuple = [previousOwner: AddressLike, newOwner: AddressLike];
  export type OutputTuple = [previousOwner: string, newOwner: string];
  export interface OutputObject {
    previousOwner: string;
    newOwner: string;
  }
  export type Event = TypedContractEvent<InputTuple, OutputTuple, OutputObject>;
  export type Filter = TypedDeferredTopicFilter<Event>;
  export type Log = TypedEventLog<Event>;
  export type LogDescription = TypedLogDescription<Event>;
}

export interface Mailbox extends BaseContract {
  connect(runner?: ContractRunner | null): Mailbox;
  waitForDeployment(): Promise<this>;

  interface: MailboxInterface;

  queryFilter<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    fromBlockOrBlockhash?: string | number | undefined,
    toBlock?: string | number | undefined
  ): Promise<Array<TypedEventLog<TCEvent>>>;
  queryFilter<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    fromBlockOrBlockhash?: string | number | undefined,
    toBlock?: string | number | undefined
  ): Promise<Array<TypedEventLog<TCEvent>>>;

  on<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    listener: TypedListener<TCEvent>
  ): Promise<this>;
  on<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    listener: TypedListener<TCEvent>
  ): Promise<this>;

  once<TCEvent extends TypedContractEvent>(
    event: TCEvent,
    listener: TypedListener<TCEvent>
  ): Promise<this>;
  once<TCEvent extends TypedContractEvent>(
    filter: TypedDeferredTopicFilter<TCEvent>,
    listener: TypedListener<TCEvent>
  ): Promise<this>;

  listeners<TCEvent extends TypedContractEvent>(
    event: TCEvent
  ): Promise<Array<TypedListener<TCEvent>>>;
  listeners(eventName?: string): Promise<Array<Listener>>;
  removeAllListeners<TCEvent extends TypedContractEvent>(
    event?: TCEvent
  ): Promise<this>;

  addOrUpdateWrapper: TypedContractMethod<
    [wrapperChainId: BytesLike, wrapper: AddressLike],
    [void],
    "nonpayable"
  >;

  initialize: TypedContractMethod<[], [void], "nonpayable">;

  messages: TypedContractMethod<[arg0: BytesLike], [string], "view">;

  nexusAppId: TypedContractMethod<[], [string], "view">;

  owner: TypedContractMethod<[], [string], "view">;

  receiveMessage: TypedContractMethod<
    [
      chainblockNumber: BigNumberish,
      receipt: MailboxMessageStruct,
      proof: BytesLike
    ],
    [void],
    "nonpayable"
  >;

  renounceOwnership: TypedContractMethod<[], [void], "nonpayable">;

  sendMessage: TypedContractMethod<
    [
      nexusAppIDTo: BytesLike[],
      to: AddressLike[],
      nonce: BigNumberish,
      data: BytesLike
    ],
    [void],
    "nonpayable"
  >;

  transferOwnership: TypedContractMethod<
    [newOwner: AddressLike],
    [void],
    "nonpayable"
  >;

  verifiedReceipts: TypedContractMethod<
    [arg0: BytesLike],
    [
      [string, string, string, bigint] & {
        nexusAppIDFrom: string;
        data: string;
        from: string;
        nonce: bigint;
      }
    ],
    "view"
  >;

  verifierWrappers: TypedContractMethod<[arg0: BytesLike], [string], "view">;

  getFunction<T extends ContractMethod = ContractMethod>(
    key: string | FunctionFragment
  ): T;

  getFunction(
    nameOrSignature: "addOrUpdateWrapper"
  ): TypedContractMethod<
    [wrapperChainId: BytesLike, wrapper: AddressLike],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "initialize"
  ): TypedContractMethod<[], [void], "nonpayable">;
  getFunction(
    nameOrSignature: "messages"
  ): TypedContractMethod<[arg0: BytesLike], [string], "view">;
  getFunction(
    nameOrSignature: "nexusAppId"
  ): TypedContractMethod<[], [string], "view">;
  getFunction(
    nameOrSignature: "owner"
  ): TypedContractMethod<[], [string], "view">;
  getFunction(
    nameOrSignature: "receiveMessage"
  ): TypedContractMethod<
    [
      chainblockNumber: BigNumberish,
      receipt: MailboxMessageStruct,
      proof: BytesLike
    ],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "renounceOwnership"
  ): TypedContractMethod<[], [void], "nonpayable">;
  getFunction(
    nameOrSignature: "sendMessage"
  ): TypedContractMethod<
    [
      nexusAppIDTo: BytesLike[],
      to: AddressLike[],
      nonce: BigNumberish,
      data: BytesLike
    ],
    [void],
    "nonpayable"
  >;
  getFunction(
    nameOrSignature: "transferOwnership"
  ): TypedContractMethod<[newOwner: AddressLike], [void], "nonpayable">;
  getFunction(
    nameOrSignature: "verifiedReceipts"
  ): TypedContractMethod<
    [arg0: BytesLike],
    [
      [string, string, string, bigint] & {
        nexusAppIDFrom: string;
        data: string;
        from: string;
        nonce: bigint;
      }
    ],
    "view"
  >;
  getFunction(
    nameOrSignature: "verifierWrappers"
  ): TypedContractMethod<[arg0: BytesLike], [string], "view">;

  getEvent(
    key: "CallbackFailed"
  ): TypedContractEvent<
    CallbackFailedEvent.InputTuple,
    CallbackFailedEvent.OutputTuple,
    CallbackFailedEvent.OutputObject
  >;
  getEvent(
    key: "Initialized"
  ): TypedContractEvent<
    InitializedEvent.InputTuple,
    InitializedEvent.OutputTuple,
    InitializedEvent.OutputObject
  >;
  getEvent(
    key: "MailboxEvent"
  ): TypedContractEvent<
    MailboxEventEvent.InputTuple,
    MailboxEventEvent.OutputTuple,
    MailboxEventEvent.OutputObject
  >;
  getEvent(
    key: "OwnershipTransferred"
  ): TypedContractEvent<
    OwnershipTransferredEvent.InputTuple,
    OwnershipTransferredEvent.OutputTuple,
    OwnershipTransferredEvent.OutputObject
  >;

  filters: {
    "CallbackFailed(address,bytes)": TypedContractEvent<
      CallbackFailedEvent.InputTuple,
      CallbackFailedEvent.OutputTuple,
      CallbackFailedEvent.OutputObject
    >;
    CallbackFailed: TypedContractEvent<
      CallbackFailedEvent.InputTuple,
      CallbackFailedEvent.OutputTuple,
      CallbackFailedEvent.OutputObject
    >;

    "Initialized(uint64)": TypedContractEvent<
      InitializedEvent.InputTuple,
      InitializedEvent.OutputTuple,
      InitializedEvent.OutputObject
    >;
    Initialized: TypedContractEvent<
      InitializedEvent.InputTuple,
      InitializedEvent.OutputTuple,
      InitializedEvent.OutputObject
    >;

    "MailboxEvent(bytes32,bytes32[],bytes,address,address[],uint256)": TypedContractEvent<
      MailboxEventEvent.InputTuple,
      MailboxEventEvent.OutputTuple,
      MailboxEventEvent.OutputObject
    >;
    MailboxEvent: TypedContractEvent<
      MailboxEventEvent.InputTuple,
      MailboxEventEvent.OutputTuple,
      MailboxEventEvent.OutputObject
    >;

    "OwnershipTransferred(address,address)": TypedContractEvent<
      OwnershipTransferredEvent.InputTuple,
      OwnershipTransferredEvent.OutputTuple,
      OwnershipTransferredEvent.OutputObject
    >;
    OwnershipTransferred: TypedContractEvent<
      OwnershipTransferredEvent.InputTuple,
      OwnershipTransferredEvent.OutputTuple,
      OwnershipTransferredEvent.OutputObject
    >;
  };
}
