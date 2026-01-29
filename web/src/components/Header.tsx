import React, { useState } from "react";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
// import Popover from "@mui/material/Popover"; // TODO:
import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";

import AboutDialog from "./AboutDialog";

export default function Header() {
  // const classes = useStyles();
  // const [anchorEl, setAnchorEl] = React.useState<HTMLElement | null>(null);

  // const handlePopoverOpen = (event: React.MouseEvent<HTMLElement, MouseEvent>) => {
  //   setAnchorEl(event.currentTarget);
  // };

  // const handlePopoverClose = () => {
  //   setAnchorEl(null);
  // };

  // const noteOpen = Boolean(anchorEl);

  const [aboutOpen, setAboutOpen] = useState(false);

  return (
    <header>
      <Typography variant="h3" component="h1" gutterBottom>
        zhconv-rs 中文简繁及地區詞轉換
      </Typography>
      <Box display="flex" alignItems="center">
        <Typography variant="h6" component="h2" gutterBottom>
          {
            "Convert Chinese between Trad, Simp and regional variants / 轉換中文简、繁體以及兩岸四地和新马的地區詞"
          }
        </Typography>
        <Box
          // // aria-owns={noteOpen ? 'note-popover' : undefined}
          // // aria-haspopup="true"
          // // onMouseEnter={handlePopoverOpen}
          // // onMouseLeave={handlePopoverClose}
          sx={{ marginBottom: "0.35em" }}
        >
          <Tooltip title="Show notes">
            <IconButton aria-label="about" onClick={() => setAboutOpen(true)}>
              <InfoOutlinedIcon color="primary" />
            </IconButton>
          </Tooltip>
        </Box>
        {/* <Popover
          id="note-popover"
          className={classes.popover}
          classes={{
            paper: classes.paper,
          }}
          open={noteOpen}
          anchorEl={anchorEl}
          anchorOrigin={{
            vertical: 'bottom',
            horizontal: 'left',
          }}
          transformOrigin={{
            vertical: 'top',
            horizontal: 'left',
          }}
          onClose={handlePopoverClose}
          disableRestoreFocus
        >
          Note / 說明
          <Typography gutterBottom>
            All the conversion rules including built-in conversion tables and CGroups comes from Chinese Wikipedia and MediaWiki, who build and maintain those rules. The app is not meant for 100% accuracy. And it is predictable to have some wrong conversions.
          </Typography>
          <Typography gutterBottom>
            包括內建轉換表和公共轉換組在內的所有字詞轉換規則均來自中文維基百科或 MediaWiki，並由後二者維護。此轉換程序不可能100%的準確性，且可預期地會包含錯誤轉換。
          </Typography>
        </Popover>*/}
      </Box>
      <AboutDialog open={aboutOpen} setOpen={setAboutOpen} />
      {/* <Typography  component="h3" gutterBottom>
        {"Based on conversion tables maintained by Chinese Wikipedia / 基於中文維基百科維護的轉換規則"}
      </Typography> */}
    </header>
  );
}
