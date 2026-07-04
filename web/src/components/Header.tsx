import React, { useState } from "react";
import Typography from "@mui/material/Typography";
import Box from "@mui/material/Box";
import InfoOutlinedIcon from "@mui/icons-material/InfoOutlined";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";

import AboutDialog from "./AboutDialog";

export default function Header() {
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
        <Box sx={{ marginBottom: "0.35em" }}>
          <Tooltip title="Show notes">
            <IconButton aria-label="about" onClick={() => setAboutOpen(true)}>
              <InfoOutlinedIcon color="primary" />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>
      <AboutDialog open={aboutOpen} setOpen={setAboutOpen} />
    </header>
  );
}