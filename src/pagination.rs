use crate::config::{BloomBotEmbed, TERMS_PER_PAGE};
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, CreateEmbed};

pub trait PageRow {
  fn title(&self) -> String;
  fn body(&self) -> String;
}

pub type PageRowRef<'a> = &'a (dyn PageRow + Send + Sync);

pub struct Pagination<'a> {
  page_data: Vec<PaginationPage<'a>>,
  page_count: usize,
  title: String,
}

impl<'a> Pagination<'a> {
  pub async fn new(
    title: impl ToString,
    entries: Vec<&'a (dyn PageRow + Send + Sync)>,
  ) -> Result<Pagination<'_>> {
    let entries_count = entries.len();
    let page_count = match entries_count == 0 {
      true => 1,
      false => (entries_count as f64 / TERMS_PER_PAGE as f64).ceil() as usize,
    };

    let page_data = match entries_count == 0 {
      true => {
        vec![PaginationPage {
          entries: vec![],
          page_number: 0,
          page_count: 1,
        }]
      }
      false => entries
        .chunks(TERMS_PER_PAGE)
        .enumerate()
        .map(|(page_number, entries)| PaginationPage {
          entries: entries.to_vec(),
          page_number,
          page_count,
        })
        .collect(),
    };

    Ok(Self {
      title: title.to_string(),
      page_data,
      page_count,
    })
  }

  pub fn get_page_count(&self) -> usize {
    self.page_count
  }

  pub fn get_last_page_number(&self) -> usize {
    // We can do this unchecked because we use entries.is_empty on instantiation
    self.page_count - 1
  }

  pub fn get_page(&self, page: usize) -> Option<&PaginationPage> {
    self.page_data.get(page)
  }

  pub fn update_page_number(&self, current_page: usize, change_by: isize) -> usize {
    let mut new_page = current_page as isize + change_by;

    if new_page < 0 {
      new_page = (self.page_count - 1) as isize;
    } else if new_page >= self.page_count as isize {
      new_page = 0;
    }

    new_page as usize
  }

  pub fn create_page_embed(&self, page: usize) -> CreateEmbed {
    let mut embed = BloomBotEmbed::new();
    let page = self.get_page(page);

    match page {
      Some(page) => {
        // If it is a valid page that is empty, it must be page 0
        // This implies that there are no terms in the glossary
        if page.is_empty() {
          embed.title(self.title.to_string());
          embed.description("No entries have been added yet.");

          embed
        } else {
          page.to_embed(&mut embed, self.title.as_str()).clone()
        }
      }
      // This should never happen unless we have a bug in our pagination code
      None => {
        embed.title(self.title.to_string());
        embed.description("This page does not exist.");

        embed
      }
    }
  }
}

pub struct PaginationPage<'a> {
  entries: Vec<&'a (dyn PageRow + Send + Sync)>,
  page_number: usize,
  page_count: usize,
}

impl PaginationPage<'_> {
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  pub fn to_embed<'embed_lifetime>(
    &'embed_lifetime self,
    embed: &'embed_lifetime mut CreateEmbed,
    title: &str,
  ) -> &'embed_lifetime mut serenity::CreateEmbed {
    embed.title(title);
    embed.description(format!(
      "Showing entries {} to {}.",
      (self.page_number * TERMS_PER_PAGE) + 1,
      (self.page_number * TERMS_PER_PAGE) + self.entries.len()
    ));

    let fields: Vec<(String, String, bool)> = self
      .entries
      .iter()
      .map(|entry| (entry.title(), entry.body(), false))
      .collect();
    embed.fields(fields);

    embed.footer(|f| {
      f.text(format!(
        "Page {} of {}",
        self.page_number + 1,
        self.page_count
      ))
    });

    embed
  }
}
