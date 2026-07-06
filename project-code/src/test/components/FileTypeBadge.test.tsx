import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { FileTypeBadge } from "../../components/media/FileTypeBadge";

describe("FileTypeBadge", () => {
  it("shows PDF label for a .pdf filename", () => {
    render(<FileTypeBadge filename="document.pdf" />);
    expect(screen.getByText("PDF")).toBeInTheDocument();
  });

  it("shows ZIP label for a .zip filename", () => {
    render(<FileTypeBadge filename="archive.zip" />);
    expect(screen.getByText("ZIP")).toBeInTheDocument();
  });

  it("shows DOCX label for a .docx filename", () => {
    render(<FileTypeBadge filename="report.docx" />);
    expect(screen.getByText("DOCX")).toBeInTheDocument();
  });

  it("shows VCF label for a .vcf filename", () => {
    render(<FileTypeBadge filename="contact.vcf" />);
    expect(screen.getByText("VCF")).toBeInTheDocument();
  });

  it("shows uppercased extension as label for unknown extension", () => {
    render(<FileTypeBadge filename="mystery.xyz" />);
    expect(screen.getByText("XYZ")).toBeInTheDocument();
  });

  it("shows uppercased filename as label when there is no dot separator", () => {
    // split(".").pop() on "noextension" returns the whole string, so label = "NOEXTENSION"
    render(<FileTypeBadge filename="noextension" />);
    expect(screen.getByText("NOEXTENSION")).toBeInTheDocument();
  });

  it("tagExt overrides the filename extension for labelling", () => {
    // filename says .txt but tagExt says pdf → should show PDF
    render(<FileTypeBadge filename="document.txt" tagExt="pdf" />);
    expect(screen.getByText("PDF")).toBeInTheDocument();
  });

  it("renders a span with file-type-badge class", () => {
    const { container } = render(<FileTypeBadge filename="doc.pdf" />);
    expect(container.querySelector(".file-type-badge")).toBeInTheDocument();
  });

  it("renders an icon span", () => {
    const { container } = render(<FileTypeBadge filename="doc.pdf" />);
    expect(container.querySelector(".file-type-badge-icon")).toBeInTheDocument();
  });

  it("renders the label span", () => {
    const { container } = render(<FileTypeBadge filename="doc.pdf" />);
    expect(container.querySelector(".file-type-badge-label")).toBeInTheDocument();
  });

  it("applies a background color via inline style", () => {
    const { container } = render(<FileTypeBadge filename="doc.pdf" />);
    const badge = container.querySelector(".file-type-badge") as HTMLElement;
    expect(badge.style.background).toBeTruthy();
  });

  it("shows TIFF label for .tif extension", () => {
    render(<FileTypeBadge filename="scan.tif" />);
    expect(screen.getByText("TIFF")).toBeInTheDocument();
  });

  it("shows TIFF label for .tiff extension", () => {
    render(<FileTypeBadge filename="scan.tiff" />);
    expect(screen.getByText("TIFF")).toBeInTheDocument();
  });
});
