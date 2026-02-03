import React from "react";
import ToggleButton from "@mui/material/ToggleButton";
import ToggleButtonGroup from "@mui/material/ToggleButtonGroup";
import Tooltip from "@mui/material/Tooltip";
import { useTheme } from "@mui/material/styles";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useApp } from "../AppContext";

export type RulesetMode = "mediawiki" | "opencc" | "both";

const RulesetSelector: React.FC<{ disabled: boolean }> = ({ disabled }) => {
  const { rulesetMode, setRulesetMode } = useApp();
  const theme = useTheme();
  const isMobile = useMediaQuery(theme.breakpoints.down('md'));
  const tooltipPlacement = isMobile ? "top" : "left";

  const handleChange = (
    _event: React.MouseEvent<HTMLElement>,
    newMode: RulesetMode | null,
  ) => {
    if (newMode !== null) {
      setRulesetMode(newMode);
    }
  };

  return (
    <ToggleButtonGroup
      value={rulesetMode}
      exclusive
      onChange={handleChange}
      aria-label="ruleset mode"
      size="small"
      disabled={disabled}
      color="primary"
      orientation={isMobile ? "horizontal" : "vertical"}
    >
      <ToggleButton value="mediawiki" aria-label="MediaWiki">
        <Tooltip
          placement={tooltipPlacement}
          disableInteractive
          title={
            <>
              Use MediaWiki conversion tables only
              <br />/ 僅使用 MediaWiki 轉換表
            </>
          }
        >
          <span>MediaWiki</span>
        </Tooltip>
      </ToggleButton>
      <ToggleButton value="opencc" aria-label="OpenCC">
        <Tooltip
          placement={tooltipPlacement}
          disableInteractive
          title={
            <>
              Use OpenCC conversion tables only
              <br />/ 僅使用 OpenCC 轉換表
            </>
          }
        >
          <span>OpenCC</span>
        </Tooltip>
      </ToggleButton>
      <ToggleButton value="both" aria-label="Both">
        <Tooltip
          placement={tooltipPlacement}
          disableInteractive
          title={
            <>
              Use both MediaWiki & OpenCC conversion tables
              <br />/ 使用 MediaWiki & OpenCC 轉換表
            </>
          }
        >
          <span>MW & OC</span>
        </Tooltip>
      </ToggleButton>
    </ToggleButtonGroup>
  );
};

export default RulesetSelector;
