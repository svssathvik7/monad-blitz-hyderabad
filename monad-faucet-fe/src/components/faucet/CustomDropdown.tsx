import React, { useState, useRef, useEffect } from "react";
import { Token } from "../../store/TokenListStore";
import { ChevronDown, ChevronUp } from "lucide-react";

type CustomDropdownProps = {
  tokens: Token[];
  selectedToken: Token | undefined;
  onSelect: (token: Token) => void;
};

const CustomDropdown: React.FC<CustomDropdownProps> = ({
  tokens,
  selectedToken,
  onSelect,
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  return (
    <div ref={dropdownRef} style={{ position: "relative", width: 180 }}>
      <button
        onClick={() => setIsOpen((prev) => !prev)}
        style={{
          width: "100%",
          padding: "8px 12px",
          borderRadius: 6,
          border: "1px solid #ccc",
          background: "#fff",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          fontSize: 15,
          cursor: "pointer",
        }}
      >
        <span style={{ display: "flex", alignItems: "center" }}>
          {selectedToken ? (
            <>
              <img
                src={selectedToken.logo_url}
                alt={selectedToken.name}
                style={{
                  width: 20,
                  height: 20,
                  borderRadius: "50%",
                  marginRight: 8,
                  objectFit: "cover",
                }}
              />
              {selectedToken.symbol}
            </>
          ) : (
            <span style={{ color: "#888" }}>Select a token</span>
          )}
        </span>
        {isOpen ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
      </button>
      {isOpen && (
        <div
          style={{
            position: "absolute",
            top: "110%",
            left: 0,
            width: "100%",
            background: "#fff",
            border: "1px solid #eee",
            borderRadius: 6,
            boxShadow: "0 2px 8px rgba(0,0,0,0.07)",
            zIndex: 10,
            maxHeight: 180,
            overflowY: "auto",
          }}
        >
          {tokens.map((token) => (
            <div
              key={token.address}
              onClick={() => {
                onSelect(token);
                setIsOpen(false);
              }}
              style={{
                display: "flex",
                alignItems: "center",
                padding: "8px 12px",
                cursor: "pointer",
                fontSize: 15,
                borderBottom: "1px solid #f5f5f5",
                background:
                  selectedToken?.address === token.address ? "#f9f9f9" : "#fff",
              }}
            >
              <img
                src={token.logo_url}
                alt={token.name}
                style={{
                  width: 20,
                  height: 20,
                  borderRadius: "50%",
                  marginRight: 8,
                  objectFit: "cover",
                }}
              />
              <span style={{ fontWeight: 500 }}>{token.symbol}</span>
              <span style={{ marginLeft: 8, color: "#888", fontSize: 13 }}>
                {token.name}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};

export default CustomDropdown;
