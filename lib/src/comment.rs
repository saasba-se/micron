use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::{Collectable, CollectableAt, Identifiable};
use crate::{Database, Result, User};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Owner of the comment is the user who published it.
    pub owner: Uuid,

    /// Parent can be anything that can be referenced with a uuid. This is the
    /// item that the comment belongs to.
    pub parent: Uuid,

    /// If the comment is a reply to another comment, this is where we specify
    /// the id of the parent comment.
    pub is_reply: Option<Uuid>,

    /// Content is just plain text. Depending on application it might be
    /// markdown or even html.
    pub content: String,

    pub published_time: DateTime<Utc>,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            owner: Uuid::nil(),
            parent: Uuid::nil(),
            is_reply: None,
            content: "".to_string(),
            published_time: Utc::now(),
        }
    }
}

impl Collectable for Comment {
    fn get_collection_name() -> &'static str {
        "comments"
    }
}

impl CollectableAt for Comment
where
    Comment: Collectable,
{
    fn get_collection_name_at(keyset: Uuid) -> String {
        format!("{}_{}", keyset, Self::get_collection_name())
    }
}

impl Identifiable for Comment {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

/// Define premade storage interface for `Comment`s. This is done as comments
/// are stored as separate collections per parent id for quicker retrieval.
impl Comment
where
    Comment: Collectable + CollectableAt + Identifiable,
{
    pub fn store_at(&self, parent: Uuid, db: &Database) -> Result<()> {
        db.set_at(Self::get_collection_name_at(parent), self)
    }

    pub fn restore_at(&self, parent: Uuid, db: &Database) -> Result<Self> {
        db.get_at(&Self::get_collection_name_at(parent), self.get_id())
    }

    pub fn remove_at(&self, parent: Uuid, db: &Database) -> Result<()> {
        db.remove_at(&Self::get_collection_name_at(parent), self)
    }

    pub fn collection_at(parent: Uuid, db: &Database) -> Result<Vec<Self>> {
        db.get_collection_at(&Self::get_collection_name_at(parent))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentNode {
    pub inner: Comment,
    pub children: Vec<CommentNode>,

    pub author: User,
    pub published_time: String,
}

pub fn comment_count(parent: Uuid, db: &Database) -> Result<usize> {
    Ok(Comment::collection_at(parent, db)?.len())
}

pub fn comment_count_user(user_id: Uuid, db: &Database) -> Result<usize> {
    let mut count = 0;
    let trees = db.trees_for::<Comment>()?;
    for tree in trees {
        count += db.get_collection_at::<Comment>(tree)?.len();
    }

    Ok(count)
}

/// Formats a comment tree based on the comments found per specified parent.
/// Attaches additional information to each comment node for convenience.
///
/// Returns a tuple of vec of nodes and total comment count.
// TODO: implementation is rather ugly currently, should use recursion.
pub fn comment_tree(parent: Uuid, db: &Database) -> Result<(Vec<CommentNode>, usize)> {
    // Get the relevant comments. Those are the ones where parent matches the
    // id we're given.
    let comments = Comment::collection_at(parent, db)?;

    let mut nodes = Vec::new();
    let mut comment_count = 0;

    let reply_comments = comments
        .iter()
        .filter(|c| c.is_reply.is_some())
        .cloned()
        .collect::<Vec<_>>();

    // Create the tree of comments, going down at most 4 layers deep
    for comment in comments {
        // Get top level comments that are not replies
        if comment.is_reply.is_none() {
            nodes.push(CommentNode {
                inner: comment.to_owned(),
                author: db.get::<User>(comment.owner)?,
                published_time: comment.published_time.format("%d-%m-%Y").to_string(),
                children: {
                    comment_count += 1;
                    reply_comments
                        .iter()
                        .filter(|c0| c0.is_reply.unwrap() == comment.id)
                        .map(|c0| CommentNode {
                            inner: c0.to_owned(),
                            author: db.get::<User>(c0.owner).unwrap(),
                            published_time: c0.published_time.format("%d-%m-%Y").to_string(),
                            children: {
                                comment_count += 1;
                                reply_comments
                                    .iter()
                                    .filter(|c1| c1.is_reply.unwrap() == c0.id)
                                    .map(|c1| CommentNode {
                                        inner: c1.to_owned(),
                                        author: db.get::<User>(c1.owner).unwrap(),
                                        published_time: c1
                                            .published_time
                                            .format("%d-%m-%Y")
                                            .to_string(),
                                        children: {
                                            comment_count += 1;
                                            reply_comments
                                                .iter()
                                                .filter(|c2| c2.is_reply.unwrap() == c1.id)
                                                .map(|c2| CommentNode {
                                                    inner: c2.to_owned(),
                                                    author: db.get::<User>(c2.owner).unwrap(),
                                                    published_time: c2
                                                        .published_time
                                                        .format("%d-%m-%Y")
                                                        .to_string(),
                                                    children: Vec::new(),
                                                })
                                        }
                                        .collect::<Vec<_>>(),
                                    })
                            }
                            .collect::<Vec<_>>(),
                        })
                }
                .collect::<Vec<_>>(),
            });
        }
    }

    Ok((nodes, comment_count))
}
