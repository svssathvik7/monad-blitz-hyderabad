import clsx, { ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { API } from "../constants/api";
import {
  DeployResponse,
  ResponseStatus,
  TransferResponse,
} from "../components/common/models";
import { Token } from "../store/TokenListStore";

export const cn = (...classes: ClassValue[]) => twMerge(clsx(classes));

const getIpAddress = async () => {
  const response = await fetch("https://api.ipify.org?format=json");
  const data = await response.json();
  return data.ip;
};

function getCode() {
  const queryString = window.location.search;
  const urlParams = new URLSearchParams(queryString);
  const codeParam = urlParams.get("code");
  return codeParam;
}

const delay = async (time_in_ms: number) => {
  return new Promise((res) => {
    setTimeout(res, time_in_ms);
  });
};

async function handleQuenchTokens(
  address: string,
  token: Token,
  setLoading: (loading: boolean) => void,
  setError: (error: boolean) => void
): Promise<TransferResponse> {
  const tick = performance.now();
  if (!address || !token) {
    return {
      status: ResponseStatus.Error,
      error: {
        message: "Invalid address or token",
      },
    };
  }
  setError(false);
  setLoading(true);
  try {
    const response = await fetch(API().withdrawToken, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${localStorage.getItem("userToken")}`,
      },
      body: JSON.stringify({
        token_address: token.address,
        to: address,
        token_type: token.token_type,
        magnification: 1,
        ip: await getIpAddress(),
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: TransferResponse = await response.json();

    if (data) {
      const tock = performance.now();
      if (tock - tick < 500) {
        await delay(500 - (tock - tick));
      }
      setLoading(false);
      return data;
    }
    throw new Error("Unexpected response format");
  } catch (error) {
    setLoading(false);
    setError(true);
    return {
      status: ResponseStatus.Error,
      error: {
        message: error instanceof Error ? error.message : String(error),
      },
    };
  }
}

async function deployToken(
  name: string,
  symbol: string,
  total_supply: string,
  decimals: string,
  deployer_address: string,
  tokenIcon: File
): Promise<DeployResponse> {
  const tick = performance.now();

  const data = {
    name,
    symbol,
    total_supply,
    decimals: Number(decimals),
    deployer_address,
    ip: await getIpAddress(),
  };

  const formData = new FormData();
  formData.append("data", JSON.stringify(data));
  formData.append("file", tokenIcon);

  try {
    const response = await fetch(API().deployToken, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${localStorage.getItem("userToken")}`,
      },
      body: formData,
    });

    if (!response.ok) {
      // setCursorState({ cursor: true, loader: false, dancer: false });
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: DeployResponse = await response.json();

    if (data) {
      const tock = performance.now();
      if (tock - tick < 500) {
        await delay(500 - (tock - tick));
      }
      // setLoading(false)
      return data;
    }

    return {
      error: "Unexpected response format",
    };
  } catch (error) {
    // setLoading(false)
    // setError(true)
    return {
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function convertToLocalTime(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString(undefined, {
    hour12: true,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export const isValidEVMAddress = (address: string) =>
  /^0x[a-fA-F0-9]{40}$/.test(address);

export {
  handleQuenchTokens,
  getCode,
  deployToken,
  convertToLocalTime,
};
