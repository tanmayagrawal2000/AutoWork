import discord
from discord.ext import commands
import os
import subprocess
import bot_config

# Path to the data directory relative to this script
DATA_DIR = os.path.join(os.path.dirname(__file__), "..", "data")
STATE_FILE = os.path.join(DATA_DIR, "state.json")

class BranchSelect(discord.ui.Select):
    def __init__(self, branches):
        options = []
        for b in branches[:25]:  # Discord limit is 25 options
            is_current = b.startswith('* ')
            branch_name = b.replace('* ', '', 1)
            desc = "Currently active branch" if is_current else f"Switch to {branch_name}"
            options.append(discord.SelectOption(
                label=branch_name, 
                description=desc, 
                default=is_current,
                emoji="📌" if is_current else None
            ))
        super().__init__(placeholder='Select a branch to switch to...', min_values=1, max_values=1, options=options)

    async def callback(self, interaction: discord.Interaction):
        branch = self.values[0]
        project_root = os.path.join(os.path.dirname(__file__), "..")
        
        try:
            process = subprocess.Popen(
                ["git", "checkout", branch],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            stdout, stderr = process.communicate()
            
            embed = interaction.message.embeds[0]
            if process.returncode == 0:
                embed.set_footer(text=f"✅ Successfully switched to branch: {branch}")
            else:
                embed.set_footer(text=f"❌ Failed to switch to: {branch}")
                
            await interaction.response.edit_message(embed=embed, view=DashboardView())
        except Exception as e:
            embed = interaction.message.embeds[0]
            embed.set_footer(text=f"⚠️ Error executing git command: {e}")
            await interaction.response.edit_message(embed=embed, view=DashboardView())

class BranchSelectView(discord.ui.View):
    def __init__(self, branches):
        super().__init__()
        self.add_item(BranchSelect(branches))
        
        # Add a Cancel button to return to the main dashboard
        cancel_btn = discord.ui.Button(label="Cancel", style=discord.ButtonStyle.secondary, emoji="❌")
        async def cancel_callback(interaction: discord.Interaction):
            embed = interaction.message.embeds[0]
            embed.set_footer(text="Action cancelled.")
            await interaction.response.edit_message(embed=embed, view=DashboardView())
        cancel_btn.callback = cancel_callback
        self.add_item(cancel_btn)


class DashboardView(discord.ui.View):
    def __init__(self):
        super().__init__(timeout=None) # Persistent view

    @discord.ui.button(label="Reset State", style=discord.ButtonStyle.danger, custom_id="btn_reset", emoji="🗑️")
    async def reset_state(self, interaction: discord.Interaction, button: discord.ui.Button):
        embed = interaction.message.embeds[0]
        if os.path.exists(STATE_FILE):
            try:
                os.remove(STATE_FILE)
                embed.set_footer(text="✅ state.json deleted. Next run requires fresh login.")
            except Exception as e:
                embed.set_footer(text=f"❌ Failed to delete state.json: {e}")
        else:
            embed.set_footer(text="⚠️ state.json does not exist. Already clean.")
            
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Run Scraper", style=discord.ButtonStyle.success, custom_id="btn_run", emoji="▶️")
    async def run_scraper(self, interaction: discord.Interaction, button: discord.ui.Button):
        script_name = "run_scraper.bat" if os.name == 'nt' else "run_scraper.sh"
        script_path = os.path.join(os.path.dirname(__file__), "..", "scripts", script_name)
        embed = interaction.message.embeds[0]
        
        try:
            subprocess.Popen(
                [script_path, "--headless"],
                cwd=os.path.join(os.path.dirname(__file__), ".."),
                creationflags=subprocess.CREATE_NEW_CONSOLE if os.name == 'nt' else 0
            )
            embed.set_footer(text="🚀 Scraper started in the background...")
        except Exception as e:
            embed.set_footer(text=f"❌ Failed to start scraper: {e}")
            
        await interaction.response.edit_message(embed=embed, view=self)

    @discord.ui.button(label="Switch Branch", style=discord.ButtonStyle.primary, custom_id="btn_branch", emoji="🔀")
    async def switch_branch(self, interaction: discord.Interaction, button: discord.ui.Button):
        project_root = os.path.join(os.path.dirname(__file__), "..")
        embed = interaction.message.embeds[0]
        
        try:
            process = subprocess.Popen(
                ["git", "branch"],
                cwd=project_root,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                text=True
            )
            stdout, _ = process.communicate()
            if process.returncode == 0 and stdout:
                branches = [b.strip() for b in stdout.split('\n') if b.strip()]
                if not branches:
                    embed.set_footer(text="⚠️ No branches found.")
                    await interaction.response.edit_message(embed=embed, view=self)
                    return
                
                embed.set_footer(text="Select a branch from the dropdown below:")
                await interaction.response.edit_message(embed=embed, view=BranchSelectView(branches))
            else:
                embed.set_footer(text="❌ Failed to fetch branches.")
                await interaction.response.edit_message(embed=embed, view=self)
        except Exception as e:
            embed.set_footer(text=f"⚠️ Error: {e}")
            await interaction.response.edit_message(embed=embed, view=self)


class AutoWorkBot(commands.Bot):
    def __init__(self):
        intents = discord.Intents.default()
        intents.message_content = True
        super().__init__(command_prefix="!", intents=intents)

    async def setup_hook(self):
        # Add the persistent view so buttons work after restarts
        self.add_view(DashboardView())

    async def on_ready(self):
        print(f"Logged in as {self.user} (ID: {self.user.id})")
        print("------")
        
        # Fetch the designated channel
        channel = self.get_channel(bot_config.CHANNEL_ID)
        
        if channel:
            # Optionally, purge old dashboard messages sent by the bot
            try:
                print("Cleaning up old bot messages in the dashboard channel...")
                async for message in channel.history(limit=50):
                    if message.author == self.user:
                        await message.delete()
            except discord.Forbidden:
                print("Note: Bot lacks 'Manage Messages' permission to clean up old messages.")
            except Exception as e:
                print(f"Failed to clear old messages: {e}")
                
            # Send the new dashboard
            embed = discord.Embed(
                title="⚙️ AutoWork Control Panel",
                description="Use the buttons below to manage the Workday scraper.",
                color=discord.Color.blurple()
            )
            await channel.send(embed=embed, view=DashboardView())
            print("Dashboard sent successfully!")
        else:
            print(f"WARNING: Could not find channel with ID {bot_config.CHANNEL_ID}. Make sure the bot is in the server and the ID is correct.")

if __name__ == "__main__":
    if bot_config.DISCORD_TOKEN == "your_bot_token_here":
        print("Please configure your DISCORD_TOKEN and CHANNEL_ID in discord_bot/bot_config.py")
    else:
        bot = AutoWorkBot()
        bot.run(bot_config.DISCORD_TOKEN)
