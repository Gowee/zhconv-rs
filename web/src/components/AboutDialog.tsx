import React from "react";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import MuiDialogTitle from "@mui/material/DialogTitle";
import MuiDialogContent from "@mui/material/DialogContent";
import MuiDialogActions from "@mui/material/DialogActions";
import IconButton from "@mui/material/IconButton";
import CloseIcon from "@mui/icons-material/Close";
import Typography from "@mui/material/Typography";
// import Divider from "@mui/material/Divider";
import Link from "@mui/material/Link";
import GitHubIcon from "@mui/icons-material/GitHub";

export interface DialogTitleProps {
  id: string;
  children: React.ReactNode;
  onClose: () => void;
}

function DialogTitle({ id, children, onClose }: DialogTitleProps) {
  return (
    <MuiDialogTitle
      id={id}
      disableTypography
      sx={{ m: 0, p: 2, position: "relative" }}
    >
      <Typography variant="h6">{children}</Typography>
      {onClose ? (
        <IconButton
          aria-label="close"
          onClick={onClose}
          sx={{
            position: "absolute",
            right: 8,
            top: 8,
            color: (theme) => theme.palette.grey[500],
          }}
        >
          <CloseIcon />
        </IconButton>
      ) : null}
    </MuiDialogTitle>
  );
}

export default function AboutDialog({
  open,
  setOpen,
}: {
  open: boolean;
  setOpen: (open: boolean) => void;
}) {
  const handleClose = () => {
    setOpen(false);
  };

  return (
    <div>
      <Dialog
        onClose={handleClose}
        aria-labelledby="about-dialog-title"
        open={open}
      >
        <DialogTitle id="about-dialog-title" onClose={handleClose}>
          Notes / 說明
        </DialogTitle>
        <MuiDialogContent dividers sx={{ p: 2 }}>
          <Typography gutterBottom>
            <Link href="https://github.com/Gowee/zhconv-rs" target="_blank" rel="noopener">
              <GitHubIcon sx={{ fontSize: "1rem" }} />
              {" zhconv-rs"}
            </Link>
            &nbsp;completes conversion in the browser with no data transmitting out. 
            Built-in conversion rulesets including built-in conversion tables
            and CGroups are sourced from MediaWiki & OpenCC and maintained by
            their communities. The accuracy is generally acceptable while
            limited, and erroneous conversions are to be expected.
          </Typography>
          <Typography gutterBottom>
            轉換均在瀏覽器內完成，數據不會向外傳輸。
            包括內建轉換表（
            <Link href="https://github.com/wikimedia/mediawiki/blob/master/includes/languages/Data/ZhConversion.php#L14">
              1
            </Link>
            、
            <Link href="https://github.com/BYVoid/OpenCC/tree/master/data/dictionary">
              2
            </Link>
            ） 和
            <Link href="https://zh.wikipedia.org/wiki/Wikipedia:%E5%AD%97%E8%A9%9E%E8%BD%89%E6%8F%9B%E8%99%95%E7%90%86/%E5%85%AC%E5%85%B1%E8%BD%89%E6%8F%9B%E7%B5%84">
              公共轉換組
            </Link>
            在內的所有字詞轉換規則均來自中文維基百科（MediaWiki）和OpenCC，並由社群維護。轉換準確性尚可，但仍可預期地會包含錯誤轉換。
          </Typography>
          <Typography gutterBottom>
            {"See also / 另见 "}
            <Link href="https://zh.wikipedia.org/wiki/Help:%E9%AB%98%E7%BA%A7%E5%AD%97%E8%AF%8D%E8%BD%AC%E6%8D%A2%E8%AF%AD%E6%B3%95">
              Help:高级字词转换语法
            </Link>
            {", "}
            <Link href="https://zh.wikipedia.org/wiki/Help:中文维基百科的繁简、地区词处理#转换技术">
              Help:中文维基百科的繁简、地区词处理#转换技术
            </Link>
            .
          </Typography>
        </MuiDialogContent>
        <MuiDialogActions sx={{ m: 0, p: 1 }}>
          <Button autoFocus onClick={handleClose} color="secondary">
            Ok / 好
          </Button>
        </MuiDialogActions>
      </Dialog>
    </div>
  );
}
