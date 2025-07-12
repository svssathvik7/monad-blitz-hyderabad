import clsx, { ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";
import { codeState, updateUser, User } from "../store/store";
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

async function verifyCaptcha(captchaToken: string) {
  const response = await fetch(API().captcha, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      ip_address: await getIpAddress(),
      token: captchaToken,
    }),
  });

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  const data: { data: { success: boolean } } = await response.json();

  return data.data.success;
}

function handleLogin() {
  localStorage.removeItem("userToken");
  const client_id = API().client_Id;
  const url = "https://github.com/login/oauth/authorize";
  const params = `client_id=${client_id}`;
  window.location.assign(`${url}?${params}`);
}

function getCode() {
  const queryString = window.location.search;
  const urlParams = new URLSearchParams(queryString);
  const codeParam = urlParams.get("code");
  return codeParam;
}

async function tokenToUser(token: string): Promise<User | undefined> {
  try {
    const userResponse = await fetch(API().user, {
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });

    if (!userResponse.ok) {
      throw new Error("Failed to fetch user data");
    }

    const data: { data: User } = await userResponse.json();

    updateUser(data.data);
    return data.data;
  } catch (error) {
    console.error("Error fetching user data:", error);
    localStorage.clear();
    updateUser(null);
  }
}

const delay = async (time_in_ms: number) => {
  return new Promise((res) => {
    setTimeout(res, time_in_ms);
  });
};

function clearQueryParams() {
  const { pathname } = window.location;
  window.history.replaceState({}, "", pathname);
}

async function fetchUser({ code, setCode }: codeState) {
  const userToken = localStorage.getItem("userToken");

  if (userToken) {
    return await tokenToUser(userToken);
  }

  if (!code) return null;

  const tokenResponse = await fetch(API().auth(code));

  if (!tokenResponse.ok) {
    throw new Error(`Failed to fetch token: ${tokenResponse.statusText}`);
  }

  const { data } = await tokenResponse.json();

  if (!data || !data.token) {
    throw new Error("Token not found in response data");
  }

  const token = data.token;

  localStorage.setItem("userToken", token);

  clearQueryParams();
  setCode(null);

  return await tokenToUser(token);
}

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

export const linkDiscord = () =>
  window.open("https://discord.gg/dZwSjh9922", "_blank", "noopener,noreferrer");

export const linkGardenInvite = () =>
  window.open("https://discord.gg/dZwSjh9922", "_blank", "noopener,noreferrer");

export const linkGarden = () =>
  window.open("https://app.garden.finance/", "_blank", "noopener,noreferrer");

export const isValidEVMAddress = (address: string) =>
  /^0x[a-fA-F0-9]{40}$/.test(address);

export {
  verifyCaptcha,
  handleLogin,
  handleQuenchTokens,
  getCode,
  fetchUser,
  deployToken,
  convertToLocalTime,
};
