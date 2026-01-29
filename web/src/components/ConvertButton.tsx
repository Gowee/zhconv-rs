import React, {
  useEffect,
  useState,
  useRef,
  forwardRef,
  ForwardedRef,
} from "react";
import Grid from "@mui/material/Grid";
import Button from "@mui/material/Button";
import ButtonGroup from "@mui/material/ButtonGroup";
import ArrowDropDownIcon from "@mui/icons-material/ArrowDropDown";
import ClickAwayListener from "@mui/material/ClickAwayListener";
import Grow from "@mui/material/Grow";
import Paper from "@mui/material/Paper";
import Popper from "@mui/material/Popper";
import MenuItem from "@mui/material/MenuItem";
import MenuList from "@mui/material/MenuList";
import Box from "@mui/material/Box";
import Tooltip from "@mui/material/Tooltip";

import PACKAGE from "../../package.json";

const variants = {
  zh: "zh 原文",
  "zh-Hant": "zh-Hant 繁體",
  "zh-Hans": "zh-Hans 简体",
  "zh-TW": "zh-TW 臺灣正體",
  "zh-HK": "zh-HK 香港繁體",
  "zh-MO": "zh-MO 澳門繁體",
  "zh-CN": "zh-CN 大陆简体",
  "zh-SG": "zh-SG 新加坡简体",
  "zh-MY": "zh-MY 大马简体",
};
type Variant = keyof typeof variants;

function ConvertButton(
  {
    onConvert: handleConvert,
  }: {
    onConvert: (target: Variant) => void;
  },
  ref: ForwardedRef<any>
) {
  const isMounting = useRef(true);
  const [open, setOpen] = useState(false);
  const anchorRef = useRef<HTMLDivElement>(null);
  const [selectedVariant, setSelectedVariant] = useState<Variant>(() => {
    const hash = window.location.hash.slice(1) as Variant;
    if (variants[hash]) {
      return hash;
    } else {
      return (
        (localStorage.getItem(`${PACKAGE.name}-selected-variant`) as Variant) ??
        "zh"
      );
    }
  });

  const handleClick = () => {
    handleConvert(selectedVariant);
  };

  const handleMenuItemClick = (
    event: React.MouseEvent<HTMLLIElement, MouseEvent>,
    variant: string
  ) => {
    setSelectedVariant(variant as Variant);
    setOpen(false);
  };

  useEffect(() => {
    if (isMounting.current) {
      isMounting.current = false;
      return;
    }
    handleConvert(selectedVariant);
    localStorage.setItem(`${PACKAGE.name}-selected-variant`, selectedVariant);
    window.history.replaceState({}, "", `#${selectedVariant}`);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedVariant]);

  const handleToggle = () => {
    setOpen((prevOpen) => !prevOpen);
  };

  const handleClose = (event: React.MouseEvent<Document, MouseEvent>) => {
    if (
      anchorRef.current &&
      anchorRef.current.contains(event.target as HTMLElement)
    ) {
      return;
    }

    setOpen(false);
  };

  return (
    <Grid container direction="column" alignItems="center">
      <Grid item xs={12}>
        <ButtonGroup
          variant="outlined"
          color="primary"
          ref={anchorRef}
          aria-label="convert button"
        >
          <Tooltip title="Click to convert / 點擊以轉換">
            <Button ref={ref} onClick={handleClick}>
              <small>To/至</small>
              &nbsp;
              <Box sx={{ whiteSpace: "nowrap" }}>
                {variants[selectedVariant]}
              </Box>
            </Button>
          </Tooltip>
          <Tooltip title="Click and select a target variant / 點擊並選擇目標變體">
            <Button
              color="primary"
              size="small"
              aria-controls={open ? "convert-button-menu" : undefined}
              aria-expanded={open ? "true" : undefined}
              aria-label="convert to selected variant"
              aria-haspopup="menu"
              onClick={handleToggle}
            >
              <ArrowDropDownIcon />
            </Button>
          </Tooltip>
        </ButtonGroup>
        <Popper
          open={open}
          anchorEl={anchorRef.current}
          role={undefined}
          style={{ zIndex: 1900 }}
          transition
          disablePortal
        >
          {({ TransitionProps, placement }) => (
            <Grow
              {...TransitionProps}
              style={{
                transformOrigin:
                  placement === "bottom" ? "center top" : "center bottom",
              }}
            >
              <Paper>
                <ClickAwayListener onClickAway={handleClose}>
                  <MenuList id="convert-button-menu">
                    {Object.entries(variants).map(([variant, option]) => (
                      <MenuItem
                        key={variant}
                        selected={selectedVariant === variant}
                        onClick={(event) => handleMenuItemClick(event, variant)}
                        style={{ justifyContent: "center" }}
                      >
                        {option}
                      </MenuItem>
                    ))}
                  </MenuList>
                </ClickAwayListener>
              </Paper>
            </Grow>
          )}
        </Popper>
      </Grid>
    </Grid>
  );
}

export default forwardRef(ConvertButton);
