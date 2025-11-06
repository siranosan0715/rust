use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use std::env;

# [derive(Debug)]
pub struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

# [poise::command(slash_command, prefix_command)]
async fn hello(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Hello!! this bot makes language is **RUST!!**").await?;
    Ok(())
}

# [poise::command(slash_command, prefix_command)]
async fn bot(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This bot is **RUSTテストbot**").await?;
    Ok(())
}

# [poise::command(slash_command, prefix_command)]
async fn name(ctx: Context<'_>) -> Result<(), Error> {
    let user = ctx.author();
    let name = &user.name;
    let user_id = user.id.get();
    let response = format!(
        "Hi!! **{}** \nyour Discord ID is  `{}` ",
        name,
        user_id
    );
    ctx.say(response).await?;
    Ok(())
}

# [poise::command(slash_command, prefix_command)]
async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let ping_duration = ctx.ping().await;

    let ping_ms = ping_duration.as_millis();

    let response = format!(
        "Websocket Heartbeat: **{}ms**",
        ping_ms
    );

    ctx.say(response).await?;
    Ok(())
}

/// ユーザーが持つロールをすべて表示します (DMでは使用できません)
#[poise::command(slash_command, prefix_command, guild_only)]
async fn roles(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.say("このコマンドはDMでは実行できません。").await?;
            return Ok(());
        }
    };
    
    // 1. ギルドメンバー情報を取得（ロール情報を含む）
    let member = match ctx.author_member().await {
        Some(member) => member,
        None => {
            ctx.say("メンバー情報（ロールを含む）が取得できませんでした。\nインテントの設定を確認してください。").await?;
            return Ok(());
        }
    };
    
    // 2. ギルドの全ロール情報を取得
    // ❌ .map(|guild| guild.roles) を 
    // ✅ .map(|guild| guild.roles.clone()) に修正
    let roles_map = ctx.serenity_context().cache.guild(guild_id)
        .map(|guild| guild.roles.clone()) // <--- ここを修正
        .unwrap_or_default();

    // メンバーが持つロールIDからロール名を取得
    let mut role_names: Vec<String> = member.roles
        .iter()
        .filter_map(|role_id| {
            roles_map.get(role_id).map(|role| role.name.clone())
        })
        .collect();

    // @everyoneロールは常に持っているので追加
    if !role_names.contains(&"@everyone".to_string()) {
        role_names.insert(0, "@everyone".to_string());
    }
    
    // 3. 応答メッセージを作成
    let user_name = ctx.author().name.clone();
    
    // @everyoneだけの場合、リストの長さは1です
    if role_names.len() <= 1 { 
        ctx.say(format!("ユーザー **{}** は、@everyone ロール以外のサーバー固有のロールを持っていません。", user_name)).await?;
    } else {
        let roles_list = role_names.join(", ");
        let response = format!(
            "ユーザー **{}** が持つロールは以下の通りです:\n{}",
            user_name,
            roles_list
        );
        ctx.say(response).await?;
    }

    Ok(())
}

# [tokio::main]
async fn main() {
    dotenv().expect("no settings '.env'");
    
    let token = env::var("DISCORD_TOKEN")
        .expect("no settings'DISCORD_TOKEN' ");
    
    let options = poise::FrameworkOptions {
        commands: vec![hello(), bot(), name(), ping(), roles()],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            case_insensitive_commands: true,
            ..Default::default()
        },
        on_error: |error| Box::pin(on_error(error)),
        ..Default::default()
    };
    
    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .expect("not conect discord");

    if let Err(why) = client.start().await {
        eprintln!("client error: {:?}", why);
    }
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Command { error, ctx, .. } => {
            eprintln!("Error in command {}: {:?}", ctx.command().name, error);
        }
        _ => eprintln!("error: {:?}", error),
    }
}