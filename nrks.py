import os
import pandas as pd
import shutil

def remove_combined_suffix(file_path):
    """ 删除文件名中的 '_combined' 后缀 """
    if '_combined' in file_path:
        new_name = file_path.replace('_combined', '')
        os.rename(file_path, new_name)
        return new_name
    return file_path

def create_song_folder(song_name, output_dir):
    """ 创建歌曲对应的文件夹 """
    song_folder = os.path.join(output_dir, song_name)
    os.makedirs(song_folder, exist_ok=True)
    return song_folder

def split_by_difficulty(df, song_name, song_folder):
    """ 按照难度拆分数据并保存为 CSV 和 Excel 文件 """
    difficulties = ['EZ', 'HD', 'IN', 'AT']
    for difficulty in difficulties:
        # 获取该难度的成绩数据
        difficulty_data = df[df['difficulty'] == difficulty]
        if not difficulty_data.empty:
            # 删除 song_name 列
            difficulty_data = difficulty_data.drop(columns=['song_name'], errors='ignore')

            csv_file = os.path.join(song_folder, f"{song_name}_{difficulty}.csv")
            excel_file = os.path.join(song_folder, f"{song_name}_{difficulty}.xlsx")
            
            # 保存为 CSV 和 Excel
            difficulty_data.to_csv(csv_file, index=False)
            difficulty_data.to_excel(excel_file, index=False)
            print(f"Generated {difficulty} file for {song_name}: {csv_file} and {excel_file}")

def main():
    current_dir = os.getcwd()  # 当前脚本所在目录
    output_dir = os.path.join(current_dir, "generated_files")  # 生成的输出文件夹
    os.makedirs(output_dir, exist_ok=True)

    # 获取当前目录下所有 CSV 文件
    files = [f for f in os.listdir(current_dir) if f.endswith('.csv')]
    
    # 遍历每个文件
    for file_name in files:
        print(f"Processing file: {file_name}")
        
        # 删除 _combined 后缀并重命名文件
        file_path = remove_combined_suffix(file_name)
        
        # 读取 CSV 文件
        df = pd.read_csv(file_path)
        
        # 提取歌曲名
        song_name = df['song_name'].iloc[0]  # 假设所有行的歌曲名一致
        
        # 创建该歌曲的文件夹
        song_folder = create_song_folder(song_name, output_dir)
        
        # 将原始文件移到对应的文件夹中
        shutil.move(file_path, os.path.join(song_folder, file_name))
        
        # 生成一个 Excel 文件（去除 'song_name' 列）
        df_without_name = df.drop(columns=['song_name'], errors='ignore')  # 删除 'song_name' 列
        excel_file = os.path.join(song_folder, f"{song_name}.xlsx")
        df_without_name.to_excel(excel_file, index=False)
        
        # 按难度拆分并生成新的文件
        split_by_difficulty(df, song_name, song_folder)
    
    # 打包生成的文件夹
    shutil.move(output_dir, os.path.join(current_dir, "generated_files"))
    
    print("Processing and file generation completed!")

if __name__ == "__main__":
    main()
