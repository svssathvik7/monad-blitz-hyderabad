type ButtonProps = {
  children: React.ReactNode;
  handleClick: () => void;
  className?: string;
  secondary?: boolean;
  disabled?: boolean;
};

export const Button: React.FC<ButtonProps> = ({
  children,
  handleClick,
  className,
  secondary = false,
  disabled = false,
}) => {
  return (
    <button
      className={`rounded-xl border px-4 py-2 ${
        secondary
          ? "border-purple-600 bg-white text-purple-950"
          : "border-purple-600 bg-purple-950 text-white"
      } ${disabled ? "cursor-not-allowed opacity-50" : ""} ${className}`}
      onClick={!disabled ? handleClick : undefined}
      disabled={disabled}
    >
      {children}
    </button>
  );
};
