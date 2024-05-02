import re
import pandas as pd
import matplotlib.pyplot as plt

with open("logs/log_parallel.log", "r") as file:
    log = file.read()

operations = ["handshake", "get response", "storage server", "pull data", "decode", "is_cache_hit"]
data = {op: {} for op in operations}

# Parse log and populate the data dictionary
for line in log.split("\n"):
    for operation in operations:
        cache_hit_match = re.match(r".*request (\d+) cache hit.*", line)
        if cache_hit_match:
            request_id = cache_hit_match.group(1)
            data["is_cache_hit"][request_id] = 1
        cache_miss_match = re.match(r".*request (\d+) cache miss.*", line)
        if cache_miss_match:
            request_id = cache_miss_match.group(1)
            data["is_cache_hit"][request_id] = 0

        match = re.match(rf".*{operation} time for request (\d+): (\d+(?:\.\d+)?)(ms|µs|s)", line)
        if match:
            request_id, time, unit = match.groups()
            # data['is_cache_hit'][request_id] = 0
            time = float(time)
            if unit == 'µs':
                time /= 1000  # convert us to ms
            elif unit == 's':
                time *= 1000  # convert s to ms
            data[operation][request_id] = time
        

df = pd.DataFrame(data)
df = df.rename(columns={'pull data': 'Data transfer'})
df = df.rename(columns={'decode': 'Decode'})
df = df.rename(columns={'storage server': 'Server time'})
df['Network time'] = df['handshake'] + df['get response'] - df['Server time']
df.drop(columns=['handshake', 'get response'], inplace=True)
df = df[['Network time', 'Server time', 'Data transfer', 'Decode', 'is_cache_hit']]
print(df)

average_times = df.mean()[:-1][::-1]  # Exclude is_cache_hit column
fig = plt.figure(figsize=(10, 6))
plt.barh(average_times.index, average_times, color='lightcoral')
for index, value in enumerate(average_times):
    plt.text(value, index, f'{int(value)} ms', ha='left', va='center')
plt.xlabel('Average Time (ms)')
plt.ylabel('Operation')
plt.title('Average Time for Each Operation')
plt.show()
    

cache_hit_data = df[df['is_cache_hit'] == 1]
cache_miss_data = df[df['is_cache_hit'] == 0]

# Calculate average times for cache hit and cache miss separately
avg_times_cache_hit = cache_hit_data.mean()[:-1][::-1]  # Exclude is_cache_hit column
avg_times_cache_miss = cache_miss_data.mean()[:-1][::-1]  # Exclude is_cache_hit column

# 2 rows 1 column
fig, axs = plt.subplots(2, 1, figsize=(10, 6))

# Plot for cache hit
axs[0].barh(avg_times_cache_hit.index, avg_times_cache_hit, color='skyblue')
for index, value in enumerate(avg_times_cache_hit):
    axs[0].text(value, index, f'{int(value)} ms', ha='left', va='center')
axs[0].set_xlabel('Average Time (ms)')
axs[0].set_ylabel('Operation')
axs[0].set_title('Average Time for Each Operation (Cache Hit)')

# Plot for cache miss
axs[1].barh(avg_times_cache_miss.index, avg_times_cache_miss, color='lightgreen')
for index, value in enumerate(avg_times_cache_miss):
    axs[1].text(value, index, f'{int(value)} ms', ha='left', va='center')
axs[1].set_xlabel('Average Time (ms)')
axs[1].set_ylabel('Operation')
axs[1].set_title('Average Time for Each Operation (Cache Miss)')

# Set the same x-axis limits for both plots
max_value = max(avg_times_cache_hit.max(), avg_times_cache_miss.max())
axs[0].set_xlim(0, max_value)
axs[1].set_xlim(0, max_value)

# Adjust layout to prevent overlap
plt.tight_layout()

plt.show()

# Plot as a pie chart
# plt.figure(figsize=(8, 8))

# # Concatenate average time and percentage strings in autopct
# def autopct_format(pct):
#     total = sum(avg_times)
#     val = int(round(pct*total/100.0))
#     return f'{val}ms ({pct:.1f}%)'

# plt.pie(avg_times, labels=avg_times.index, autopct=autopct_format)
# plt.title('Average Time for Each Type of Operation')
# plt.show()
