import { create } from "zustand";

export type Token = {
  created_by: string;
  token_type: string;
  address: string;
  logo_url: string;
  chain_id: number;
  symbol: string;
  name: string;
  decimals: number;
};

type TokenListState = {
  tokens: Token[];
  setTokens: (tokens: Token[]) => void;
};

export const useTokenListStore = create<TokenListState>((set) => ({
  tokens: [],
  setTokens: (tokens) => set({ tokens }),
}));
