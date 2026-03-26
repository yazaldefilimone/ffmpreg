use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Default)]
pub struct Metadata {
	pub title: Option<String>,
	pub description: Option<String>,
	pub artist: Option<String>,
	pub album: Option<String>,
	pub album_artist: Option<String>,
	pub track_number: Option<u32>,
	pub tracks_total: Option<u32>,
	pub disc_number: Option<u32>,
	pub discs_total: Option<u32>,
	pub genre: Option<String>,
	pub date: Option<String>,
	pub lyrics: Option<String>,
	pub comment: Option<String>,
	pub images: Vec<AttachedImage>,
	pub raw: HashMap<String, RawValue>,
}

#[derive(Debug, Clone)]
pub struct AttachedImage {
	pub data: Vec<u8>,
	pub mime_type: String,
	pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AttachedFile {
	pub data: Vec<u8>,
	pub filename: String,
}

#[derive(Debug, Clone)]
pub enum RawValue {
	String(String),
	Bytes(Vec<u8>),
	Image(AttachedImage),
	File(AttachedFile),
	None,
}

impl Default for RawValue {
	fn default() -> Self {
		Self::None
	}
}

// temp
impl Display for AttachedImage {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(d) = &self.description {
			writeln!(f, "Description: {}", d)?;
		}
		writeln!(f, "Mime Type: {}", self.mime_type)?;
		Ok(())
	}
}
impl Display for Metadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(t) = &self.title {
			writeln!(f, "Title: {}", t)?;
		}
		if let Some(a) = &self.artist {
			writeln!(f, "Artist: {}", a)?;
		}
		if let Some(al) = &self.album {
			writeln!(f, "Album: {}", al)?;
		}
		if let Some(g) = &self.genre {
			writeln!(f, "Genre: {}", g)?;
		}
		if let Some(d) = &self.description {
			writeln!(f, "Description: {}", d)?;
		}
		if let Some(aa) = &self.album_artist {
			writeln!(f, "Album Artist: {}", aa)?;
		}
		if let Some(tn) = self.track_number {
			writeln!(f, "Track Number: {}", tn)?;
		}
		if let Some(tt) = self.tracks_total {
			writeln!(f, "Total Tracks: {}", tt)?;
		}
		if let Some(dn) = self.disc_number {
			writeln!(f, "Disc Number: {}", dn)?;
		}
		if let Some(dt) = self.discs_total {
			writeln!(f, "Total Discs: {}", dt)?;
		}
		if let Some(l) = &self.lyrics {
			writeln!(f, "Lyrics: {}", l)?;
		}
		if let Some(c) = &self.comment {
			writeln!(f, "Comment: {}", c)?;
		}
		if let Some(date) = &self.date {
			writeln!(f, "Date: {}", date)?;
		}

		for image in &self.images {
			writeln!(f, "Image: {}", image)?;
		}

		if !self.raw.is_empty() {
			writeln!(f, "Raw: {:?}", self.raw)?;
		}

		Ok(())
	}
}
