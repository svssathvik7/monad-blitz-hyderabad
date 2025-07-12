import { Button } from "../ui/Button";
import { API } from "../../constants/api";

export const JuicyTokens = () => {
  const handleTokenListClick = () => {
    window.open(API().tokens, "_blank", "noopener,noreferrer");
  };
  return (
    <div className="z-0 flex flex-col space-y-8">
      <h1 className="w-full text-center text-xl leading-[20px] sm:text-2xl">
        See the token list here
      </h1>
      <div className="flex w-full items-center justify-end">
        <Button handleClick={handleTokenListClick}>Token List</Button>
      </div>
    </div>
  );
};
