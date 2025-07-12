const REQUIRED_ENV_VARS = {
  BACKEND_URL: import.meta.env.VITE_BACKEND_URL,
  CAPTCHA_SITE_ID: import.meta.env.VITE_SITE_ID,
  GITHUB_CLIENT_ID: import.meta.env.VITE_GITHUB_CLIENT_ID,
  EXPLORER: import.meta.env.VITE_EXPLORER_URL,
} as const;

export const API = () => {
  Object.entries(REQUIRED_ENV_VARS).forEach(([key, value]) => {
    if (!value) throw new Error(`Missing ${key} in env`);
  });

  return {
    site_Id: REQUIRED_ENV_VARS.CAPTCHA_SITE_ID,
    client_Id: REQUIRED_ENV_VARS.GITHUB_CLIENT_ID,
    captcha: REQUIRED_ENV_VARS.BACKEND_URL + "/verify-turnstile-captcha",
    test_auth: REQUIRED_ENV_VARS.BACKEND_URL + "/test_auth",
    withdrawToken: REQUIRED_ENV_VARS.BACKEND_URL + "/withdraw",
    deployToken: REQUIRED_ENV_VARS.BACKEND_URL + "/deploy/erc20",
    user: REQUIRED_ENV_VARS.BACKEND_URL + "/user",
    upload: REQUIRED_ENV_VARS.BACKEND_URL + "/upload",
    tokens: REQUIRED_ENV_VARS.BACKEND_URL + "/tokens",
    explorer: REQUIRED_ENV_VARS.EXPLORER,
    auth: (code: string) =>
      REQUIRED_ENV_VARS.BACKEND_URL + `/auth?code=${code}`,
  };
};
