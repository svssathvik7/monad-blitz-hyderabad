import Lottie from "react-lottie-player";
import Spinner from "../../constants/loader.json";
import { FC } from "react";

type LoaderProps = {
  width?: number;
  height?: number;
  speed?: number;
  className?: string;
};

export const Loader: FC<LoaderProps> = ({
  width = 270,
  height = 360,
  speed = 1,
  className,
}) => {
  return (
    <Lottie
      loop
      animationData={Spinner}
      play
      speed={speed}
      className={className}
      style={{ width: width, height: height }}
    />
  );
};
