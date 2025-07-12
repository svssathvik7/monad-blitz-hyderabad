import { User } from "../../store/store";

export const enum ResponseStatus {
  Success,
  Error,
}

export type TransferResponse = {
  status?: ResponseStatus;
  error?: {
    message: string;
    next_access?: string;
  };
  data?: { tx_hash: string; amount: string; magnification: number };
};

export type DeployResponse = {
  status?: ResponseStatus;
  error?: string;
  data?: { contract_address: string };
};

export type { User };
