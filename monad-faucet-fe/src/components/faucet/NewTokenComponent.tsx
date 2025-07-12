import { FC, useState, useMemo } from "react";
import { Button } from "../ui/Button";
import { InputBox } from "../ui/InputBox";
import { deployToken, isValidEVMAddress } from "../../utils/utils";
import { useTokenListStore } from "../../store/TokenListStore";

type NewTokenComponentProps = {
  onClose: () => void;
};

export const NewTokenComponent: FC<NewTokenComponentProps> = ({ onClose }) => {
  const [tokenName, setTokenName] = useState<string>();
  const [tokenIcon, setTokenIcon] = useState<File | null>(null);
  const [tokenSymbol, setTokenSymbol] = useState<string>();
  const [tokenSupply, setTokenSupply] = useState<string>();
  const [tokenDecimals, setTokenDecimals] = useState<string>();
  const [deployerAddress, setDeployerAddress] = useState<string>();
  const [message, setMessage] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);

  const disabled = useMemo(() => {
    return (
      !tokenName ||
      !tokenSymbol ||
      !tokenSupply ||
      !tokenDecimals ||
      !tokenIcon ||
      !deployerAddress ||
      isLoading ||
      !isValidEVMAddress(deployerAddress)
    );
  }, [
    tokenName,
    tokenSymbol,
    tokenSupply,
    tokenDecimals,
    deployerAddress,
    tokenIcon,
    isLoading,
  ]);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setTokenIcon(e.target.files[0]);
    }
  };

  const handleDeployToken = async () => {
    if (
      !tokenName ||
      !tokenSymbol ||
      !tokenSupply ||
      !tokenDecimals ||
      !deployerAddress ||
      !tokenIcon
    ) {
      return;
    }

    const { tokens } = useTokenListStore.getState();
    const symbolExists = tokens.some(
      (token) => token.symbol.toUpperCase() === tokenSymbol.toUpperCase()
    );
    if (symbolExists) {
      setMessage("Error: The token symbol already exists.");
      return;
    } else {
      setMessage(null);
    }

    setIsLoading(true);
    const response = await deployToken(
      tokenName,
      tokenSymbol.toUpperCase(),
      tokenSupply,
      tokenDecimals,
      deployerAddress,
      tokenIcon
    );

    if (response.error) {
      setMessage("Deployment Error: " + response.error);
      setIsLoading(false);
    }

    if (response.data) {
      setMessage(
        "Token deployed successfully: " + response.data.contract_address
      );
      setIsLoading(false);
    }
  };

  // Prevent modal close on click inside modal
  const handleModalClick = (e: React.MouseEvent<HTMLDivElement>) => {
    e.stopPropagation();
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-40"
      onClick={onClose}
    >
      <div
        className="mx-auto flex max-w-lg flex-col gap-6 rounded-xl bg-white p-8 shadow-lg"
        style={{ minWidth: 400 }}
        onClick={handleModalClick}
      >
        <h2 className="mb-4 text-center text-2xl font-bold">
          Create New Token
        </h2>
        {message && (
          <div className="mb-2 text-center text-sm font-medium text-red-500">
            {message}
          </div>
        )}
        <form
          className="space-y-6"
          onSubmit={(e) => {
            e.preventDefault();
            handleDeployToken();
          }}
        >
          <div>
            <label className="mb-1 block text-sm font-medium">Token Name</label>
            <InputBox
              type="text"
              width="100%"
              defaultValue={tokenName}
              placeholder="Enter token name"
              onChange={(e) => setTokenName(e.target.value)}
            />
          </div>
          <div>
            <label className="mb-1 block text-sm font-medium">Token Logo</label>
            <div className="flex items-center gap-4">
              <InputBox
                imageURL={tokenIcon ? URL.createObjectURL(tokenIcon) : ""}
                width={"170px"}
                type="file"
                accept="image/*"
                defaultValue={tokenIcon?.name}
                onChange={handleFileChange}
              />
              {tokenIcon && (
                <img
                  src={URL.createObjectURL(tokenIcon)}
                  alt="Token Icon Preview"
                  className="h-12 w-12 rounded-full border"
                />
              )}
            </div>
          </div>
          <div>
            <label className="block text-sm font-medium">Token Symbol</label>
            <InputBox
              type="text"
              width="100%"
              defaultValue={tokenSymbol}
              placeholder="e.g. USDT"
              onChange={(e) => setTokenSymbol(e.target.value)}
            />
          </div>
          <div className="flex gap-4">
            <div className="flex-1">
              <label className="mb-1 block text-sm font-medium">
                Token Decimals
              </label>
              <InputBox
                type="number"
                width="100%"
                defaultValue={tokenDecimals}
                placeholder="e.g. 18"
                onChange={(e) => setTokenDecimals(e.target.value)}
              />
            </div>
            <div className="flex-1">
              <label className="mb-1 block text-sm font-medium">
                Total Supply
              </label>
              <InputBox
                type="number"
                width="100%"
                defaultValue={tokenSupply}
                placeholder="e.g. 1000000"
                onChange={(e) => setTokenSupply(e.target.value)}
              />
            </div>
          </div>
          <div>
            <label className="mb-1 mt-2 block text-sm font-medium">
              Deployer Address (receives 20%)
            </label>
            <InputBox
              type="text"
              width="100%"
              defaultValue={deployerAddress}
              placeholder="0x..."
              onChange={(e) => setDeployerAddress(e.target.value)}
            />
          </div>
          <div className="mt-6 flex gap-4">
            <Button className="w-full" secondary handleClick={onClose}>
              Cancel
            </Button>
            <Button
              className="w-full"
              handleClick={handleDeployToken}
              disabled={disabled}
            >
              {isLoading ? "Deploying..." : "Deploy Token"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
};
